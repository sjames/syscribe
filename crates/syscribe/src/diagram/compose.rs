use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

use serde::Deserialize;
use syscribe_model::element::RawElement;

use crate::diagram::layout::router::{arrowhead_defs, route_edge};
use crate::diagram::layout::{build_element_node, load_metrics, render_element, IncludeFilter, ViewConfig, ViewPreset};
use crate::diagram::solver::{
    solve_layout, PlacementCanvas, PlacementFile,
    ElementPlacement as SolverElementPlacement,
};

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

// ── Model-driven compose ──────────────────────────────────────────────────────

/// Compose a diagram SVG directly from a `Diagram` model element.
///
/// Reads `expose:` for the element list, `edges:` (array format) for connections,
/// assigns col/row positions via topology sort, runs the Cassowary solver, then
/// pipes through the compose pipeline.  Output path is resolved from `svgFile` or
/// companion convention (replace `.md` with `.svg` in the element's file path).
pub fn cmd_diagram_compose_from_model(
    elements: &[RawElement],
    diagram_qname: &str,
    output_override: Option<&str>,
) {
    let diagram = match elements.iter().find(|e| e.qualified_name == diagram_qname) {
        Some(e) => e,
        None => {
            eprintln!("error: diagram element '{}' not found", diagram_qname);
            std::process::exit(1);
        }
    };

    if !matches!(
        diagram.frontmatter.element_type.as_ref(),
        Some(syscribe_model::element::ElementType::Diagram) | None
    ) {
        // Allow if kind field implies diagram, but warn otherwise
    }

    let fm = &diagram.frontmatter;

    // Exposed element qnames
    let expose: Vec<String> = fm
        .expose
        .as_ref()
        .map(|v| v.iter().filter_map(|e| e.as_str().map(str::to_string)).collect())
        .unwrap_or_default();

    if expose.is_empty() {
        eprintln!("error: '{}' has no expose: list", diagram_qname);
        std::process::exit(1);
    }

    let kind = fm.diagram_kind.clone().unwrap_or_else(|| "arch".to_string());
    let ibd = kind == "ibd";

    // Edges from frontmatter (array format, same schema as compose layout JSON)
    let edges_json: Option<serde_json::Value> = fm
        .edges
        .as_ref()
        .and_then(|v| if v.is_sequence() { yaml_to_json(v) } else { None });

    // Assign col/row using topology sort on edges
    let col_row = topology_col_row(&expose, edges_json.as_ref());

    // Default view per diagram kind
    let default_view = if ibd {
        serde_json::Value::String("name".to_string())
    } else {
        serde_json::Value::String("full".to_string())
    };

    let placement = PlacementFile {
        title: fm.name.clone().or_else(|| fm.title.clone()),
        kind: Some(kind.clone()),
        canvas: Some(PlacementCanvas { padding: Some(40.0), bg: Some("#ffffff".to_string()) }),
        elements: expose
            .iter()
            .map(|qn| {
                let (col, row) = col_row.get(qn.as_str()).copied().unwrap_or((0, 0));
                SolverElementPlacement {
                    qname: qn.clone(),
                    col,
                    row,
                    view: Some(default_view.clone()),
                    pin_x: None,
                    pin_y: None,
                }
            })
            .collect(),
        edges: edges_json.clone(),
    };

    let resolved = solve_layout(elements, &placement);
    let layout_json = serde_json::to_string_pretty(&resolved).unwrap();

    // Write to temp file and pipe through compose
    let tmp = "/tmp/_syscribe_diagram_auto.json";
    if let Err(e) = std::fs::write(tmp, &layout_json) {
        eprintln!("error writing temp layout: {}", e);
        std::process::exit(1);
    }

    let svg_path = output_override
        .map(str::to_string)
        .unwrap_or_else(|| companion_svg_path(diagram));

    cmd_diagram_compose(elements, tmp, Some(&svg_path), ibd);
    eprintln!("wrote {}", svg_path);
}

/// Assign col/row positions using topological longest-path sort on the edge graph.
/// Elements with no incoming edges get col=0.  Row is assigned by order within each col.
/// Falls back to a square grid when there are no edges.
fn topology_col_row(expose: &[String], edges: Option<&serde_json::Value>) -> HashMap<String, (u32, u32)> {
    let expose_set: HashSet<&str> = expose.iter().map(String::as_str).collect();

    // Extract (src_qname, tgt_qname) pairs from edges JSON array
    let edge_pairs: Vec<(&str, &str)> = edges
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|e| {
                    let src = e.get("from")?.get("qname")?.as_str()?;
                    let tgt = e.get("to")?.get("qname")?.as_str()?;
                    if expose_set.contains(src) && expose_set.contains(tgt) {
                        Some((src, tgt))
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    if edge_pairs.is_empty() {
        // Grid layout: ceil(sqrt(N)) columns
        let n = expose.len();
        let n_cols = (n as f64).sqrt().ceil() as u32;
        return expose
            .iter()
            .enumerate()
            .map(|(i, qn)| (qn.clone(), (i as u32 % n_cols, i as u32 / n_cols)))
            .collect();
    }

    // Kahn's algorithm: assign col = longest path from any source
    let mut col: HashMap<&str, u32> = expose.iter().map(|q| (q.as_str(), 0)).collect();
    let mut in_deg: HashMap<&str, u32> = expose.iter().map(|q| (q.as_str(), 0)).collect();
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();

    for (src, tgt) in &edge_pairs {
        adj.entry(src).or_default().push(tgt);
        *in_deg.entry(tgt).or_insert(0) += 1;
    }

    let mut queue: VecDeque<&str> = in_deg
        .iter()
        .filter(|(_, &d)| d == 0)
        .map(|(&q, _)| q)
        .collect();

    while let Some(u) = queue.pop_front() {
        let u_col = *col.get(u).unwrap_or(&0);
        for &v in adj.get(u).map(Vec::as_slice).unwrap_or(&[]) {
            let v_col = col.entry(v).or_insert(0);
            if u_col + 1 > *v_col {
                *v_col = u_col + 1;
            }
            let deg = in_deg.entry(v).or_insert(1);
            *deg = deg.saturating_sub(1);
            if *deg == 0 {
                queue.push_back(v);
            }
        }
    }

    // Rows: order within each column by expose list order
    let mut row_counter: HashMap<u32, u32> = HashMap::new();
    expose
        .iter()
        .map(|qn| {
            let c = *col.get(qn.as_str()).unwrap_or(&0);
            let r = *row_counter.entry(c).or_insert(0);
            row_counter.insert(c, r + 1);
            (qn.clone(), (c, r))
        })
        .collect()
}

/// Derive the companion SVG path from the diagram element's file path.
fn companion_svg_path(diagram: &RawElement) -> String {
    if let Some(svg_file) = &diagram.frontmatter.svg_file {
        let dir = Path::new(&diagram.file_path)
            .parent()
            .unwrap_or(Path::new("."));
        dir.join(svg_file.trim_start_matches("./"))
            .to_string_lossy()
            .into_owned()
    } else {
        // Replace .md extension with .svg
        if diagram.file_path.ends_with(".md") {
            format!("{}svg", &diagram.file_path[..diagram.file_path.len() - 2])
        } else {
            format!("{}.svg", diagram.file_path)
        }
    }
}

/// Convert a `serde_yaml::Value` to `serde_json::Value` by round-tripping through JSON.
fn yaml_to_json(v: &serde_yaml::Value) -> Option<serde_json::Value> {
    serde_json::to_value(v).ok()
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
