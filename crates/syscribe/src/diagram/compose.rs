use serde::Deserialize;
use syscribe_model::element::RawElement;

use crate::diagram::layout::router::{arrowhead_defs, route_edge};
use crate::diagram::layout::{build_element_node, load_metrics, render_element, IncludeFilter, ViewConfig, ViewPreset};

#[derive(Debug, Deserialize)]
struct LayoutFile {
    title: Option<String>,
    canvas: Option<CanvasConfig>,
    elements: Vec<ElementPlacement>,
    edges: Option<Vec<EdgeSpec>>,
}

#[derive(Debug, Deserialize)]
struct CanvasConfig {
    padding: Option<f64>,
    #[allow(dead_code)]
    grid: Option<f64>,
    bg: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ElementPlacement {
    qname: String,
    x: f64,
    y: f64,
    /// Optional view spec: either a preset string or a full config object.
    view: Option<ViewSpec>,
}

/// Accepts `"ports"` (string) or `{ "preset": "ports", "include": { ... } }` (object).
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum ViewSpec {
    Preset(String),
    Config {
        preset: Option<String>,
        include: Option<IncludeSpec>,
        min_width: Option<f64>,
    },
}

#[derive(Debug, Clone, Deserialize)]
struct IncludeSpec {
    ports: Option<Vec<String>>,
    features: Option<Vec<String>>,
}

impl ViewSpec {
    fn into_view_config(self) -> ViewConfig {
        match self {
            ViewSpec::Preset(s) => ViewConfig::from_preset(&s),
            ViewSpec::Config { preset, include, min_width } => {
                let preset_str = preset.as_deref().unwrap_or("full");
                let inc = include.map(|i| IncludeFilter {
                    ports: i.ports,
                    features: i.features,
                }).unwrap_or_default();
                ViewConfig {
                    preset: ViewPreset::from_str(preset_str),
                    include: inc,
                    min_width,
                    ibd: false,
                }
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct EdgeSpec {
    from: EdgeEndpoint,
    to: EdgeEndpoint,
    kind: Option<String>,
    label: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EdgeEndpoint {
    #[serde(alias = "element")]
    qname: String,
    port: Option<String>,
}

struct PlacedElement {
    qname: String,
    x: f64,
    y: f64,
    svg: String,
    width: f64,
    height: f64,
    /// (port_name, abs_x, abs_y) — absolute canvas coordinates
    port_anchors: Vec<(String, f64, f64)>,
}

pub fn cmd_diagram_compose(elements: &[RawElement], layout_file: &str, output_file: Option<&str>, ibd: bool) {
    let content = match std::fs::read_to_string(layout_file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error reading layout file '{}': {}", layout_file, e);
            std::process::exit(1);
        }
    };

    let layout: LayoutFile = match serde_json::from_str(&content) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error parsing layout JSON: {}", e);
            std::process::exit(1);
        }
    };

    let metrics = load_metrics();
    let padding = layout.canvas.as_ref().and_then(|c| c.padding).unwrap_or(40.0);
    let bg = layout
        .canvas
        .as_ref()
        .and_then(|c| c.bg.clone())
        .unwrap_or_else(|| "#fafafa".to_string());

    // Render each element and record absolute port anchor positions
    let mut placed: Vec<PlacedElement> = Vec::new();

    for placement in &layout.elements {
        let elem = elements.iter().find(|e| e.qualified_name == placement.qname);
        let elem = match elem {
            Some(e) => e,
            None => {
                eprintln!("warn: element '{}' not found, skipping", placement.qname);
                continue;
            }
        };

        let mut view = placement
            .view
            .as_ref()
            .map(|v| v.clone().into_view_config())
            .unwrap_or_default();
        view.ibd = ibd;

        let node = build_element_node(elem, &view);
        let rendered = render_element(&node, metrics.as_ref());

        let abs_anchors: Vec<(String, f64, f64)> = rendered
            .port_anchors
            .iter()
            .map(|a| (a.name.clone(), placement.x + a.x, placement.y + a.y))
            .collect();

        placed.push(PlacedElement {
            qname: placement.qname.clone(),
            x: placement.x,
            y: placement.y,
            svg: rendered.svg,
            width: rendered.width,
            height: rendered.height,
            port_anchors: abs_anchors,
        });
    }

    // Canvas size
    let max_x = placed.iter().map(|p| p.x + p.width).fold(0.0_f64, f64::max);
    let max_y = placed.iter().map(|p| p.y + p.height).fold(0.0_f64, f64::max);
    let canvas_w = max_x + padding * 2.0;
    let canvas_h = max_y + padding * 2.0;

    let mut svg_parts: Vec<String> = Vec::new();

    // Title
    if let Some(title) = &layout.title {
        svg_parts.push(format!(
            "<text x=\"{x:.1}\" y=\"{y:.1}\" font-size=\"16\" font-weight=\"700\" \
             fill=\"#1a1a2e\">{title}</text>",
            x = padding,
            y = padding / 2.0 + 12.0,
            title = esc(title)
        ));
    }

    // Arrowhead marker defs
    svg_parts.push(arrowhead_defs());

    // Edges — drawn behind elements
    if let Some(edges) = &layout.edges {
        for edge in edges {
            let kind = edge.kind.as_deref().unwrap_or("flow");
            let from_xy = find_anchor(&placed, &edge.from.qname, edge.from.port.as_deref());
            let to_xy = find_anchor(&placed, &edge.to.qname, edge.to.port.as_deref());

            let (x1, y1) = from_xy.unwrap_or_else(|| {
                placed
                    .iter()
                    .find(|p| p.qname == edge.from.qname)
                    .map(|p| (p.x + p.width, p.y + p.height / 2.0))
                    .unwrap_or((0.0, 0.0))
            });
            let (x2, y2) = to_xy.unwrap_or_else(|| {
                placed
                    .iter()
                    .find(|p| p.qname == edge.to.qname)
                    .map(|p| (p.x, p.y + p.height / 2.0))
                    .unwrap_or((0.0, 0.0))
            });

            let mut edge_svg = route_edge(
                x1 + padding, y1 + padding,
                x2 + padding, y2 + padding,
                kind,
            );

            if let Some(label) = &edge.label {
                let lx = (x1 + x2) / 2.0 + padding;
                let ly = (y1 + y2) / 2.0 + padding - 6.0;
                edge_svg.push_str(&format!(
                    "<text x=\"{lx:.1}\" y=\"{ly:.1}\" font-size=\"9\" fill=\"#555\" \
                     text-anchor=\"middle\" font-style=\"italic\">{lbl}</text>",
                    lx = lx, ly = ly, lbl = esc(label)
                ));
            }

            svg_parts.push(edge_svg);
        }
    }

    // Elements — strip the <svg> wrapper from each fragment, place in <g>
    for p in &placed {
        let inner = strip_svg_wrapper(&p.svg);
        svg_parts.push(format!(
            "<a href=\"/ui/detail/{qname}\" data-qname=\"{qname}\">\
             <g transform=\"translate({x:.1} {y:.1})\">\n{inner}\n</g>\
             </a>",
            qname = esc(&p.qname),
            x = p.x + padding,
            y = p.y + padding,
            inner = inner
        ));
    }

    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" \
         viewBox=\"0 0 {w:.0} {h:.0}\" width=\"{w:.0}\" height=\"{h:.0}\">\n\
         <style>text{{font-family:'Inter','Roboto',system-ui,sans-serif}}\
         @font-face{{font-family:'Inter';font-weight:400;\
         src:url('https://fonts.bunny.net/inter/files/inter-latin-400-normal.woff2') format('woff2')}}\
         @font-face{{font-family:'Inter';font-weight:700;\
         src:url('https://fonts.bunny.net/inter/files/inter-latin-700-normal.woff2') format('woff2')}}\
         </style>\n\
         <rect width=\"{w:.0}\" height=\"{h:.0}\" fill=\"{bg}\"/>\n\
         {inner}\n\
         </svg>",
        w = canvas_w,
        h = canvas_h,
        bg = bg,
        inner = svg_parts.join("\n")
    );

    match output_file {
        Some(path) => {
            if let Err(e) = std::fs::write(path, &svg) {
                eprintln!("error writing output file '{}': {}", path, e);
                std::process::exit(1);
            }
        }
        None => println!("{}", svg),
    }
}

pub fn cmd_diagram_render(elements: &[RawElement], qname: &str, output_file: Option<&str>, view: ViewConfig) {
    let elem = elements.iter().find(|e| e.qualified_name == qname);
    let elem = match elem {
        Some(e) => e,
        None => {
            eprintln!("error: element '{}' not found", qname);
            std::process::exit(1);
        }
    };

    let metrics = load_metrics();
    let node = build_element_node(elem, &view);
    let rendered = render_element(&node, metrics.as_ref());

    match output_file {
        Some(path) => {
            if let Err(e) = std::fs::write(path, &rendered.svg) {
                eprintln!("error writing output file '{}': {}", path, e);
                std::process::exit(1);
            }
        }
        None => println!("{}", rendered.svg),
    }
}

fn find_anchor(placed: &[PlacedElement], qname: &str, port_name: Option<&str>) -> Option<(f64, f64)> {
    let elem = placed.iter().find(|p| p.qname == qname)?;
    if let Some(pname) = port_name {
        elem.port_anchors
            .iter()
            .find(|(n, _, _)| n == pname)
            .map(|(_, x, y)| (*x, *y))
    } else {
        None
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn strip_svg_wrapper(svg: &str) -> String {
    if let Some(start) = svg.find('>') {
        let after_open = &svg[start + 1..];
        if let Some(end) = after_open.rfind("</svg>") {
            return after_open[..end].trim().to_string();
        }
    }
    svg.to_string()
}
