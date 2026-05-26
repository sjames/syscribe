use std::collections::HashMap;
use serde::Deserialize;

use crate::element::RawElement;
use crate::resolver::Resolver;

// ---------------------------------------------------------------------------
// Data structures for deserialization from serde_yaml::Value
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct DiagramShape {
    #[serde(rename = "ref")]
    element_ref: String,
    kind: String,
}

#[derive(Debug, Deserialize)]
struct DiagramEdge {
    #[serde(rename = "ref")]
    #[allow(dead_code)]
    element_ref: Option<String>,
    source: String,
    target: String,
    kind: String,
}

#[derive(Debug, Deserialize)]
struct ShapeLayout {
    x: f64,
    y: f64,
    w: Option<f64>,
    h: Option<f64>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// HTML-escape `&`, `<`, `>`, `"`.
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Truncate to `max_chars` chars; append `…` if truncated.
fn truncate_str(s: &str, max_chars: usize) -> String {
    let mut chars = s.chars();
    let collected: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{}\u{2026}", collected)
    } else {
        collected
    }
}

/// Default (w, h) per element kind.
fn default_size(kind: &str) -> (f64, f64) {
    match kind {
        "RequirementDef" => (240.0, 56.0),
        "Requirement" => (240.0, 70.0),
        "TestCase" => (200.0, 56.0),
        "PartDef" | "Part" => (160.0, 46.0),
        _ => (200.0, 50.0),
    }
}

/// Compute the border-exit point on a rectangle when a ray from the rectangle
/// center heads toward `(toward_x, toward_y)`.
fn rect_border(rx: f64, ry: f64, rw: f64, rh: f64, toward_x: f64, toward_y: f64) -> (f64, f64) {
    let cx = rx + rw / 2.0;
    let cy = ry + rh / 2.0;
    let dx = toward_x - cx;
    let dy = toward_y - cy;

    if dx == 0.0 && dy == 0.0 {
        return (cx, cy);
    }

    let half_w = rw / 2.0;
    let half_h = rh / 2.0;

    let t_h = if dx != 0.0 { half_w / dx.abs() } else { f64::INFINITY };
    let t_v = if dy != 0.0 { half_h / dy.abs() } else { f64::INFINITY };
    let t = t_h.min(t_v);

    (cx + dx * t, cy + dy * t)
}

// ---------------------------------------------------------------------------
// SVG rendering helpers per shape kind
// ---------------------------------------------------------------------------

fn render_shape(
    id: &str,
    kind: &str,
    qref: &str,
    name: &str,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    is_abstract: bool,
    doc: &str,
) -> String {
    let id_e = esc(id);
    let qref_e = esc(qref);
    let name_e = esc(name);
    let cx = w / 2.0;

    match kind {
        "RequirementDef" => {
            let abstract_line = if is_abstract {
                format!(
                    "  <text x=\"{cx:.2}\" y=\"{yp:.2}\" text-anchor=\"middle\" \
                     font-size=\"9\" fill=\"#666\" font-style=\"italic\">isAbstract</text>\n",
                    cx = cx,
                    yp = h - 4.0,
                )
            } else {
                String::new()
            };
            format!(
                "<g id=\"{id}\" sysml:ref=\"{qref}\" transform=\"translate({x:.2},{y:.2})\">\n\
                 \x20 <rect width=\"{w:.2}\" height=\"{h:.2}\" rx=\"4\" fill=\"#f9f7ff\" \
                 stroke=\"#4a0a6e\" stroke-width=\"1.5\"/>\n\
                 \x20 <rect width=\"{w:.2}\" height=\"20\" rx=\"4\" ry=\"0\" fill=\"#4a0a6e\" \
                 opacity=\"0.12\"/>\n\
                 \x20 <text x=\"{cx:.2}\" y=\"14\" text-anchor=\"middle\" font-size=\"10\" \
                 fill=\"#4a0a6e\" font-style=\"italic\">\u{00ab}requirement def\u{00bb}</text>\n\
                 \x20 <text x=\"{cx:.2}\" y=\"38\" text-anchor=\"middle\" font-size=\"13\" \
                 font-weight=\"bold\" fill=\"#2d0644\">{name}</text>\n\
                 {abstract_line}</g>",
                id = id_e, qref = qref_e, x = x, y = y,
                w = w, h = h, cx = cx, name = name_e,
                abstract_line = abstract_line,
            )
        }
        "Requirement" => {
            let lines: Vec<&str> = doc
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .take(2)
                .collect();
            let line1 = lines.first().map(|s| truncate_str(s, 34)).unwrap_or_default();
            let line2 = lines.get(1).map(|s| truncate_str(s, 34)).unwrap_or_default();
            format!(
                "<g id=\"{id}\" sysml:ref=\"{qref}\" transform=\"translate({x:.2},{y:.2})\">\n\
                 \x20 <rect width=\"{w:.2}\" height=\"{h:.2}\" rx=\"4\" fill=\"#f9f7ff\" \
                 stroke=\"#4a0a6e\" stroke-width=\"1.5\"/>\n\
                 \x20 <rect width=\"{w:.2}\" height=\"18\" rx=\"4\" ry=\"0\" fill=\"#4a0a6e\" \
                 opacity=\"0.10\"/>\n\
                 \x20 <text x=\"{cx:.2}\" y=\"13\" text-anchor=\"middle\" font-size=\"9\" \
                 fill=\"#4a0a6e\" font-style=\"italic\">\u{00ab}requirement\u{00bb}</text>\n\
                 \x20 <text x=\"{cx:.2}\" y=\"32\" text-anchor=\"middle\" font-size=\"12\" \
                 font-weight=\"bold\" fill=\"#2d0644\">{name}</text>\n\
                 \x20 <text x=\"8\" y=\"50\" font-size=\"9\" fill=\"#333\">{line1}</text>\n\
                 \x20 <text x=\"8\" y=\"62\" font-size=\"9\" fill=\"#333\">{line2}</text>\n\
                 </g>",
                id = id_e, qref = qref_e, x = x, y = y,
                w = w, h = h, cx = cx, name = name_e,
                line1 = esc(&line1), line2 = esc(&line2),
            )
        }
        "TestCase" => {
            let first_line = doc
                .lines()
                .map(|l| l.trim())
                .find(|l| !l.is_empty())
                .map(|s| truncate_str(s, 38))
                .unwrap_or_default();
            format!(
                "<g id=\"{id}\" sysml:ref=\"{qref}\" transform=\"translate({x:.2},{y:.2})\">\n\
                 \x20 <rect width=\"{w:.2}\" height=\"{h:.2}\" rx=\"4\" fill=\"#f0fff4\" \
                 stroke=\"#1e6b2e\" stroke-width=\"1.5\"/>\n\
                 \x20 <rect width=\"{w:.2}\" height=\"18\" rx=\"4\" ry=\"0\" fill=\"#1e6b2e\" \
                 opacity=\"0.10\"/>\n\
                 \x20 <text x=\"{cx:.2}\" y=\"13\" text-anchor=\"middle\" font-size=\"9\" \
                 fill=\"#1e6b2e\" font-style=\"italic\">\u{00ab}test case\u{00bb}</text>\n\
                 \x20 <text x=\"{cx:.2}\" y=\"32\" text-anchor=\"middle\" font-size=\"11\" \
                 font-weight=\"bold\" fill=\"#0f3d1a\">{name}</text>\n\
                 \x20 <text x=\"{cx:.2}\" y=\"48\" text-anchor=\"middle\" font-size=\"9\" \
                 fill=\"#555\">{first_line}</text>\n\
                 </g>",
                id = id_e, qref = qref_e, x = x, y = y,
                w = w, h = h, cx = cx, name = name_e,
                first_line = esc(&first_line),
            )
        }
        "PartDef" | "Part" => {
            let stereotype = if kind == "PartDef" { "part def" } else { "part" };
            format!(
                "<g id=\"{id}\" sysml:ref=\"{qref}\" transform=\"translate({x:.2},{y:.2})\">\n\
                 \x20 <rect width=\"{w:.2}\" height=\"{h:.2}\" rx=\"4\" fill=\"#f5f5fa\" \
                 stroke=\"#3a3a4a\" stroke-width=\"1.5\"/>\n\
                 \x20 <text x=\"{cx:.2}\" y=\"16\" text-anchor=\"middle\" font-size=\"9\" \
                 fill=\"#555\">\u{00ab}{stereotype}\u{00bb}</text>\n\
                 \x20 <text x=\"{cx:.2}\" y=\"33\" text-anchor=\"middle\" font-size=\"11\" \
                 font-weight=\"bold\" fill=\"#1a1a2e\">{name}</text>\n\
                 </g>",
                id = id_e, qref = qref_e, x = x, y = y,
                w = w, h = h, cx = cx,
                stereotype = stereotype,
                name = name_e,
            )
        }
        _ => {
            format!(
                "<g id=\"{id}\" sysml:ref=\"{qref}\" transform=\"translate({x:.2},{y:.2})\">\n\
                 \x20 <rect width=\"{w:.2}\" height=\"{h:.2}\" rx=\"4\" fill=\"#f5f5fa\" \
                 stroke=\"#666\" stroke-width=\"1.2\"/>\n\
                 \x20 <text x=\"{cx:.2}\" y=\"16\" text-anchor=\"middle\" font-size=\"9\" \
                 fill=\"#666\" font-style=\"italic\">\u{00ab}{kind}\u{00bb}</text>\n\
                 \x20 <text x=\"{cx:.2}\" y=\"33\" text-anchor=\"middle\" font-size=\"11\" \
                 font-weight=\"bold\" fill=\"#222\">{name}</text>\n\
                 </g>",
                id = id_e, qref = qref_e, x = x, y = y,
                w = w, h = h, cx = cx,
                kind = esc(kind),
                name = name_e,
            )
        }
    }
}

/// Returns `(stroke_color, dasharray_value_or_empty, marker_id)`.
fn edge_style(kind: &str) -> (&'static str, &'static str, &'static str) {
    match kind {
        "derivedFrom" => ("#555", "5,3", "arr-open"),
        "verifies" => ("#3a6ea5", "", "arr-verify"),
        "allocatedTo" => ("#7a3ea5", "3,3", "arr-alloc"),
        _ => ("#888", "", "arr-open"),
    }
}

fn arrowhead_defs() -> String {
    // Build the <defs> block using format! to avoid raw-string issues with # and identifiers.
    let open_stroke = "#555";
    let verify_stroke = "#3a6ea5";
    let alloc_stroke = "#7a3ea5";
    format!(
        "<defs>\n\
         \x20 <marker id=\"arr-open\" markerWidth=\"10\" markerHeight=\"7\" \
         refX=\"9\" refY=\"3.5\" orient=\"auto\">\n\
         \x20\x20 <polyline points=\"0,0 9,3.5 0,7\" fill=\"none\" \
         stroke=\"{open}\" stroke-width=\"1.2\"/>\n\
         \x20 </marker>\n\
         \x20 <marker id=\"arr-verify\" markerWidth=\"10\" markerHeight=\"7\" \
         refX=\"9\" refY=\"3.5\" orient=\"auto\">\n\
         \x20\x20 <polyline points=\"0,0 9,3.5 0,7\" fill=\"none\" \
         stroke=\"{verify}\" stroke-width=\"1.2\"/>\n\
         \x20 </marker>\n\
         \x20 <marker id=\"arr-alloc\" markerWidth=\"10\" markerHeight=\"7\" \
         refX=\"9\" refY=\"3.5\" orient=\"auto\">\n\
         \x20\x20 <polyline points=\"0,0 9,3.5 0,7\" fill=\"none\" \
         stroke=\"{alloc}\" stroke-width=\"1.2\"/>\n\
         \x20 </marker>\n\
         </defs>",
        open = open_stroke,
        verify = verify_stroke,
        alloc = alloc_stroke,
    )
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Render a diagram `RawElement` as an SVG string.
/// Returns `None` if the element has no `layout` block.
pub fn render_diagram(
    element: &RawElement,
    _resolver: &Resolver,
    elements: &[RawElement],
) -> Option<String> {
    // Require a layout block
    let layout_val = element.frontmatter.layout.as_ref()?;

    // Parse shapes
    let shapes: HashMap<String, DiagramShape> = element
        .frontmatter
        .shapes
        .as_ref()
        .and_then(|v| serde_yaml::from_value(v.clone()).ok())
        .unwrap_or_default();

    // Parse edges
    let edges: HashMap<String, DiagramEdge> = element
        .frontmatter
        .edges
        .as_ref()
        .and_then(|v| serde_yaml::from_value(v.clone()).ok())
        .unwrap_or_default();

    // Parse layout
    let layout: HashMap<String, ShapeLayout> =
        serde_yaml::from_value(layout_val.clone()).ok().unwrap_or_default();

    // Build element lookup by qualified name
    let elem_by_qname: HashMap<&str, &RawElement> = elements
        .iter()
        .map(|e| (e.qualified_name.as_str(), e))
        .collect();

    // Compute bounding box and shape rects: id -> (x, y, w, h)
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    let mut shape_rects: HashMap<&str, (f64, f64, f64, f64)> = HashMap::new();

    for (shape_id, shape) in &shapes {
        let sl = match layout.get(shape_id.as_str()) {
            Some(l) => l,
            None => continue,
        };
        let (dw, dh) = default_size(&shape.kind);
        let w = sl.w.unwrap_or(dw);
        let h = sl.h.unwrap_or(dh);
        let x = sl.x;
        let y = sl.y;
        shape_rects.insert(shape_id.as_str(), (x, y, w, h));
        if x < min_x { min_x = x; }
        if y < min_y { min_y = y; }
        if x + w > max_x { max_x = x + w; }
        if y + h > max_y { max_y = y + h; }
    }

    if min_x == f64::INFINITY {
        min_x = 0.0;
        min_y = 0.0;
        max_x = 100.0;
        max_y = 100.0;
    }

    let pad = 20.0;
    let vx = min_x - pad;
    let vy = min_y - pad;
    let vw = (max_x - min_x) + 2.0 * pad;
    let vh = (max_y - min_y) + 2.0 * pad;

    // -----------------------------------------------------------------------
    // Render edges (drawn behind shapes)
    // -----------------------------------------------------------------------
    let mut edge_svgs: Vec<String> = Vec::new();

    // Sort edges by id for stable output
    let mut edge_ids: Vec<&String> = edges.keys().collect();
    edge_ids.sort();

    for edge_id in edge_ids {
        let edge = &edges[edge_id];
        let src_rect = match shape_rects.get(edge.source.as_str()) {
            Some(r) => *r,
            None => continue,
        };
        let tgt_rect = match shape_rects.get(edge.target.as_str()) {
            Some(r) => *r,
            None => continue,
        };

        let (sx, sy, sw, sh) = src_rect;
        let (tx, ty, tw, th) = tgt_rect;

        let tcx = tx + tw / 2.0;
        let tcy = ty + th / 2.0;
        let scx = sx + sw / 2.0;
        let scy = sy + sh / 2.0;

        let (x1, y1) = rect_border(sx, sy, sw, sh, tcx, tcy);
        let (x2, y2) = rect_border(tx, ty, tw, th, scx, scy);

        let midx = (x1 + x2) / 2.0;
        let midy = (y1 + y2) / 2.0;

        let (stroke, dash, marker) = edge_style(&edge.kind);
        let dash_attr = if dash.is_empty() {
            String::new()
        } else {
            format!(" stroke-dasharray=\"{}\"", dash)
        };

        let label = match edge.kind.as_str() {
            "derivedFrom" => "derived from",
            "verifies" => "verifies",
            "allocatedTo" => "allocated to",
            other => other,
        };

        edge_svgs.push(format!(
            "<line id=\"{id}\" x1=\"{x1:.2}\" y1=\"{y1:.2}\" x2=\"{x2:.2}\" y2=\"{y2:.2}\" \
             stroke=\"{stroke}\" stroke-width=\"1.2\"{dash} marker-end=\"url(#{marker})\"/>\n\
             <text x=\"{midx:.2}\" y=\"{midy:.2}\" font-size=\"9\" fill=\"{stroke}\" \
             text-anchor=\"middle\" dy=\"-4\">\u{00ab}{label}\u{00bb}</text>",
            id = esc(edge_id),
            x1 = x1, y1 = y1, x2 = x2, y2 = y2,
            stroke = stroke,
            dash = dash_attr,
            marker = marker,
            midx = midx, midy = midy,
            label = esc(label),
        ));
    }

    // -----------------------------------------------------------------------
    // Render shapes
    // -----------------------------------------------------------------------
    let mut shape_svgs: Vec<String> = Vec::new();

    // Sort shapes by id for stable output
    let mut shape_ids: Vec<&String> = shapes.keys().collect();
    shape_ids.sort();

    for shape_id in shape_ids {
        let shape = &shapes[shape_id];
        let (x, y, w, h) = match shape_rects.get(shape_id.as_str()) {
            Some(r) => *r,
            None => continue,
        };

        // Look up the referenced element for name, doc, isAbstract
        let (name, doc, is_abstract) = match elem_by_qname.get(shape.element_ref.as_str()) {
            Some(e) => {
                let n = e.frontmatter.name.clone().unwrap_or_else(|| {
                    e.qualified_name
                        .split("::")
                        .last()
                        .unwrap_or(&e.qualified_name)
                        .to_string()
                });
                let doc = e.doc.clone();
                let is_abstract = e.frontmatter.is_abstract.unwrap_or(false);
                (n, doc, is_abstract)
            }
            None => {
                let n = shape
                    .element_ref
                    .split("::")
                    .last()
                    .unwrap_or(&shape.element_ref)
                    .to_string();
                (n, String::new(), false)
            }
        };

        shape_svgs.push(render_shape(
            shape_id,
            &shape.kind,
            &shape.element_ref,
            &name,
            x, y, w, h,
            is_abstract,
            &doc,
        ));
    }

    // -----------------------------------------------------------------------
    // Assemble SVG
    // -----------------------------------------------------------------------
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" \
         xmlns:sysml=\"urn:syscribe:1.0\" \
         width=\"{vw:.0}\" height=\"{vh:.0}\" \
         viewBox=\"{vx:.2} {vy:.2} {vw:.2} {vh:.2}\">\n\
         {defs}\n\
         {edges}\n\
         {shapes}\n\
         </svg>",
        vw = vw, vh = vh,
        vx = vx, vy = vy,
        defs = arrowhead_defs(),
        edges = edge_svgs.join("\n"),
        shapes = shape_svgs.join("\n"),
    );

    Some(svg)
}
