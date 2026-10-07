//! IR → static SVG (`REQ-TRS-VIS-010`, spec §8.16.5).
//!
//! Draws a [`DiagramGraph`] whose every node carries a pin, and nothing
//! else: [`render_svg`] returns `None` for a graph with an unpinned node
//! because the server never computes a layout — the browser is the layout
//! authority, and its *Pin all* writes the `layout:` this writer reads.
//!
//! Pins are parent-relative (`x`/`y` relative to the parent's origin, as the
//! sprotty client writes them) with optional measured `w`/`h`; absolute
//! rectangles are computed by walking the parents, and a node without a size
//! gets one by kind (a block 160×50, a port 12×12, a container the bounding
//! box of its children plus padding). Edges follow their pinned waypoints
//! when they have any, else a straight line between the two endpoint shapes
//! clipped to their borders.
//!
//! The output follows §8.16.5: the `sysml:` namespace, a `<g id class
//! sysml:ref>` per node, a `<path id class sysml:ref sysml:source
//! sysml:target>` per edge, and the shared visual language of
//! [`super::style`] (`REQ-TRS-VIS-012`). When the `links` closure yields a
//! URL for a node's reference, that node's `<g>` is wrapped in
//! `<a xlink:href href target="_blank" rel="noopener">` exactly as
//! `REQ-TRS-LINK-002` specifies; otherwise it is left unwrapped. Output is
//! deterministic: nodes depth-first in declaration order, then edges.

use std::collections::HashMap;

use super::ir::{DiagramGraph, Edge, Node, NodeKind, Point};
use super::style::{edge_style, node_style, port_style, ArrowHead, EDGE_STROKE};

/// Margin around the drawing, in user units.
const MARGIN: f64 = 20.0;
/// Padding inside a container whose size is computed from its children.
const PADDING: f64 = 16.0;
/// Height reserved for a container's header (stereotype + name).
const HEADER: f64 = 36.0;
/// Line height of the header and compartment text.
const LINE: f64 = 14.0;
const PORT_SIZE: f64 = 12.0;

/// XML-escape text and attribute values.
pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// A number formatted without a trailing `.0` so the output stays tidy.
fn num(v: f64) -> String {
    let r = (v * 100.0).round() / 100.0;
    if r.fract() == 0.0 {
        format!("{}", r as i64)
    } else {
        format!("{r}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct AbsRect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl AbsRect {
    fn cx(&self) -> f64 {
        self.x + self.w / 2.0
    }
    fn cy(&self) -> f64 {
        self.y + self.h / 2.0
    }
    fn right(&self) -> f64 {
        self.x + self.w
    }
    fn bottom(&self) -> f64 {
        self.y + self.h
    }
}

fn is_container(node: &Node) -> bool {
    matches!(
        node.kind,
        NodeKind::Boundary | NodeKind::SystemBoundary | NodeKind::Swimlane | NodeKind::Fragment | NodeKind::State
    )
}

/// Default size for a node without a measured `w`/`h`, by kind.
fn default_size(node: &Node) -> (f64, f64) {
    match node.kind {
        NodeKind::Port => (PORT_SIZE, PORT_SIZE),
        NodeKind::Initial | NodeKind::Final => (20.0, 20.0),
        NodeKind::Choice => (24.0, 24.0),
        NodeKind::History => (20.0, 20.0),
        NodeKind::Label => (80.0, 20.0),
        NodeKind::Note => (140.0, 40.0),
        NodeKind::Actor => (40.0, 60.0),
        NodeKind::Lifeline => (120.0, 40.0),
        NodeKind::Activation => (12.0, 60.0),
        NodeKind::Requirement | NodeKind::TestCase => (180.0, 70.0),
        NodeKind::UseCase => (140.0, 50.0),
        NodeKind::Compartment => (160.0, LINE * node.lines.len().max(1) as f64 + 6.0),
        _ => (160.0, 50.0),
    }
}

/// The size of a node: its measured pin size, else — for a container with
/// children — the box bounding those children (relative coordinates) plus
/// padding, else the kind's default.
fn size_of(graph: &DiagramGraph, node: &Node, depth: usize) -> (f64, f64) {
    if let Some(pin) = node.pin {
        if let (Some(w), Some(h)) = (pin.w, pin.h) {
            return (w, h);
        }
    }
    let (dw, dh) = default_size(node);
    if (is_container(node) || node.kind == NodeKind::Block) && depth < graph.nodes.len() {
        let mut right: f64 = 0.0;
        let mut bottom: f64 = 0.0;
        let mut any = false;
        for child in graph.children_of(&node.id) {
            let Some(p) = child.pin else { continue };
            let (cw, ch) = size_of(graph, child, depth + 1);
            right = right.max(p.x + cw);
            bottom = bottom.max(p.y + ch);
            any = true;
        }
        if any {
            return ((right + PADDING).max(dw), (bottom + PADDING).max(HEADER + PADDING).max(dh));
        }
    }
    (dw, dh)
}

/// The absolute origin of a node: its own pin plus every ancestor's.
fn origin_of(graph: &DiagramGraph, node: &Node) -> (f64, f64) {
    let mut x = 0.0;
    let mut y = 0.0;
    let mut cur = Some(node);
    let mut steps = 0;
    while let Some(n) = cur {
        if let Some(p) = n.pin {
            x += p.x;
            y += p.y;
        }
        cur = n.parent.as_deref().and_then(|p| graph.node(p));
        steps += 1;
        if steps > graph.nodes.len() {
            break;
        }
    }
    (x, y)
}

/// Where a straight line from `from`'s centre towards `towards` leaves
/// `from`'s border.
fn clip(from: &AbsRect, towards: (f64, f64)) -> (f64, f64) {
    let (cx, cy) = (from.cx(), from.cy());
    let (dx, dy) = (towards.0 - cx, towards.1 - cy);
    if dx.abs() < f64::EPSILON && dy.abs() < f64::EPSILON {
        return (cx, cy);
    }
    let tx = if dx.abs() < f64::EPSILON { f64::INFINITY } else { (from.w / 2.0) / dx.abs() };
    let ty = if dy.abs() < f64::EPSILON { f64::INFINITY } else { (from.h / 2.0) / dy.abs() };
    let t = tx.min(ty);
    (cx + dx * t, cy + dy * t)
}

/// The point halfway along a polyline.
fn midpoint(pts: &[(f64, f64)]) -> (f64, f64) {
    let total: f64 = pts.windows(2).map(|w| ((w[1].0 - w[0].0).powi(2) + (w[1].1 - w[0].1).powi(2)).sqrt()).sum();
    let mut remaining = total / 2.0;
    for w in pts.windows(2) {
        let len = ((w[1].0 - w[0].0).powi(2) + (w[1].1 - w[0].1).powi(2)).sqrt();
        if len >= remaining || len == 0.0 {
            let f = if len == 0.0 { 0.0 } else { remaining / len };
            return (w[0].0 + (w[1].0 - w[0].0) * f, w[0].1 + (w[1].1 - w[0].1) * f);
        }
        remaining -= len;
    }
    pts.last().copied().unwrap_or((0.0, 0.0))
}

fn marker_id(head: ArrowHead) -> Option<&'static str> {
    match head {
        ArrowHead::None => None,
        ArrowHead::Filled => Some("arrow-filled"),
        ArrowHead::Open => Some("arrow-open"),
        ArrowHead::HollowTriangle => Some("arrow-inherit"),
        ArrowHead::FilledDiamond => Some("arrow-composition"),
        ArrowHead::HollowDiamond => Some("arrow-aggregation"),
        ArrowHead::FilledCircle => Some("arrow-circle"),
    }
}

fn marker_def(id: &str) -> String {
    let (w, h, path, fill) = match id {
        "arrow-filled" => (10, 8, "M 0,0 L 10,4 L 0,8 z", EDGE_STROKE),
        "arrow-open" => (10, 8, "M 0,0 L 10,4 L 0,8", "none"),
        "arrow-inherit" => (12, 10, "M 0,0 L 12,5 L 0,10 z", "#fff"),
        "arrow-composition" => (14, 8, "M 0,4 L 7,0 L 14,4 L 7,8 z", EDGE_STROKE),
        "arrow-aggregation" => (14, 8, "M 0,4 L 7,0 L 14,4 L 7,8 z", "#fff"),
        "arrow-circle" => (8, 8, "M 4,0 A 4,4 0 1,0 4,8 A 4,4 0 1,0 4,0 z", EDGE_STROKE),
        _ => unreachable!("marker ids are the closed set above"),
    };
    format!(
        "    <marker id=\"{id}\" viewBox=\"0 0 {w} {h}\" refX=\"{w}\" refY=\"{ry}\" markerWidth=\"{w}\" markerHeight=\"{h}\" markerUnits=\"userSpaceOnUse\" orient=\"auto-start-reverse\"><path d=\"{path}\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"1.2\"/></marker>\n",
        ry = num(h as f64 / 2.0),
        stroke = EDGE_STROKE
    )
}

fn text_el(x: f64, y: f64, anchor: &str, size: f64, extra: &str, s: &str) -> String {
    format!(
        "    <text x=\"{}\" y=\"{}\" text-anchor=\"{anchor}\" font-size=\"{}\" font-family=\"Helvetica, Arial, sans-serif\"{extra}>{}</text>\n",
        num(x),
        num(y),
        num(size),
        escape(s)
    )
}

fn node_class(node: &Node) -> String {
    let mut c = node.kind.as_str().to_string();
    if let Some(t) = &node.element_type {
        c.push(' ');
        c.push_str(t);
    }
    if node.kind == NodeKind::Port {
        if let Some(d) = node.direction {
            c.push(' ');
            c.push_str(d.as_str());
        }
    }
    if node.is_abstract {
        c.push_str(" abstract");
    }
    if !node.resolved {
        c.push_str(" unresolved");
    }
    c
}

fn draw_node(node: &Node, r: &AbsRect, out: &mut String) {
    out.push_str(&format!(
        "  <g id=\"{}\" class=\"{}\" sysml:ref=\"{}\">\n",
        escape(&node.id),
        escape(&node_class(node)),
        escape(&node.element_ref)
    ));
    let style = node_style(node);
    let dash = if style.dashed { " stroke-dasharray=\"4,3\"" } else { "" };
    let text_color = &style.text;
    match node.kind {
        NodeKind::Port => {
            let ps = port_style(node.direction);
            out.push_str(&format!(
                "    <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"1.2\"{dash}/>\n",
                num(r.x),
                num(r.y),
                num(r.w),
                num(r.h),
                ps.fill,
                ps.stroke
            ));
            if ps.glyph == "inout" {
                out.push_str(&format!(
                    "    <path d=\"M {},{} L {},{} L {},{} z\" fill=\"{}\"/>\n",
                    num(r.x),
                    num(r.bottom()),
                    num(r.right()),
                    num(r.y),
                    num(r.right()),
                    num(r.bottom()),
                    ps.stroke
                ));
            }
            out.push_str(&text_el(r.x, r.bottom() + 10.0, "start", 9.0, &format!(" fill=\"{text_color}\""), &node.label));
        }
        NodeKind::Label => {
            out.push_str(&text_el(r.x, r.y + 12.0, "start", 11.0, &format!(" fill=\"{text_color}\""), &node.label));
        }
        NodeKind::Initial | NodeKind::Final | NodeKind::History | NodeKind::Choice => {
            let (cx, cy, rad) = (r.cx(), r.cy(), r.w.min(r.h) / 2.0);
            match node.kind {
                NodeKind::Initial => out.push_str(&format!("    <circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"#222\"/>\n", num(cx), num(cy), num(rad))),
                NodeKind::Final => out.push_str(&format!(
                    "    <circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"#fff\" stroke=\"#222\" stroke-width=\"1.2\"/>\n    <circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"#222\"/>\n",
                    num(cx), num(cy), num(rad), num(cx), num(cy), num(rad * 0.6)
                )),
                NodeKind::History => out.push_str(&format!(
                    "    <circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"#fff\" stroke=\"#222\" stroke-width=\"1.2\"/>\n{}",
                    num(cx), num(cy), num(rad), text_el(cx, cy + 4.0, "middle", 10.0, " fill=\"#222\"", "H")
                )),
                _ => out.push_str(&format!(
                    "    <path d=\"M {},{} L {},{} L {},{} L {},{} z\" fill=\"#fff\" stroke=\"#222\" stroke-width=\"1.2\"/>\n",
                    num(cx), num(r.y), num(r.right()), num(cy), num(cx), num(r.bottom()), num(r.x), num(cy)
                )),
            }
        }
        NodeKind::Compartment => {
            out.push_str(&format!(
                "    <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"1\"{dash}/>\n",
                num(r.x),
                num(r.y),
                num(r.w),
                num(r.h),
                style.stroke
            ));
            let mut y = r.y + LINE - 2.0;
            for l in &node.lines {
                out.push_str(&text_el(r.x + 6.0, y, "start", 10.0, &format!(" fill=\"{text_color}\""), l));
                y += LINE;
            }
        }
        _ => {
            let rounded = matches!(node.kind, NodeKind::Boundary | NodeKind::State | NodeKind::UseCase | NodeKind::SystemBoundary);
            let rx = if rounded { " rx=\"8\"" } else { "" };
            out.push_str(&format!(
                "    <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"{rx} fill=\"{}\" stroke=\"{}\" stroke-width=\"1.4\"{dash}/>\n",
                num(r.x),
                num(r.y),
                num(r.w),
                num(r.h),
                style.fill,
                style.stroke
            ));
            let header_lines = node.stereotype.iter().count() + node.banners.len() + 1;
            let header_h = LINE * header_lines as f64 + 4.0;
            if let Some(hf) = &style.header_fill {
                out.push_str(&format!(
                    "    <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"{rx} fill=\"{hf}\" fill-opacity=\"0.15\" stroke=\"none\"/>\n",
                    num(r.x),
                    num(r.y),
                    num(r.w),
                    num(header_h.min(r.h))
                ));
            }
            let cx = r.cx();
            let mut y = r.y + LINE - 1.0;
            if let Some(st) = &node.stereotype {
                out.push_str(&text_el(cx, y, "middle", 10.0, &format!(" fill=\"{text_color}\""), &format!("«{st}»")));
                y += LINE;
            }
            for b in &node.banners {
                out.push_str(&text_el(cx, y, "middle", 10.0, &format!(" fill=\"{text_color}\""), &format!("«{b}»")));
                y += LINE;
            }
            let italic = if node.is_abstract { " font-style=\"italic\"" } else { "" };
            out.push_str(&text_el(cx, y, "middle", 12.0, &format!(" font-weight=\"bold\"{italic} fill=\"{text_color}\""), &node.label));
            y += LINE;
            if !node.lines.is_empty() {
                out.push_str(&format!(
                    "    <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"1\"/>\n",
                    num(r.x),
                    num(y - 10.0),
                    num(r.right()),
                    num(y - 10.0),
                    style.stroke
                ));
                for l in &node.lines {
                    out.push_str(&text_el(r.x + 6.0, y, "start", 10.0, &format!(" fill=\"{text_color}\""), l));
                    y += LINE;
                }
            }
        }
    }
    out.push_str("  </g>\n");
}

fn draw_edge(edge: &Edge, rects: &HashMap<&str, AbsRect>, shift: (f64, f64), used: &mut Vec<&'static str>, out: &mut String) {
    let (Some(s), Some(t)) = (rects.get(edge.source.as_str()), rects.get(edge.target.as_str())) else { return };
    let style = edge_style(edge.kind);
    let inner: Vec<(f64, f64)> = edge
        .waypoints
        .as_ref()
        .map(|pts| pts.iter().map(|Point { x, y }| (x + shift.0, y + shift.1)).collect())
        .unwrap_or_default();
    let first_towards = inner.first().copied().unwrap_or((t.cx(), t.cy()));
    let last_towards = inner.last().copied().unwrap_or((s.cx(), s.cy()));
    let mut pts = vec![clip(s, first_towards)];
    pts.extend(inner);
    pts.push(clip(t, last_towards));
    let d = pts
        .iter()
        .enumerate()
        .map(|(i, (x, y))| format!("{} {},{}", if i == 0 { "M" } else { "L" }, num(*x), num(*y)))
        .collect::<Vec<_>>()
        .join(" ");
    let mut attrs = String::new();
    if let Some(r) = &edge.element_ref {
        attrs.push_str(&format!(" sysml:ref=\"{}\"", escape(r)));
    }
    attrs.push_str(&format!(" sysml:source=\"{}\" sysml:target=\"{}\"", escape(&edge.source), escape(&edge.target)));
    if let Some(dash) = &style.dash {
        attrs.push_str(&format!(" stroke-dasharray=\"{dash}\""));
    }
    if let Some(m) = marker_id(style.arrow_target) {
        attrs.push_str(&format!(" marker-end=\"url(#{m})\""));
        if !used.contains(&m) {
            used.push(m);
        }
    }
    if let Some(m) = marker_id(style.arrow_source) {
        attrs.push_str(&format!(" marker-start=\"url(#{m})\""));
        if !used.contains(&m) {
            used.push(m);
        }
    }
    out.push_str(&format!(
        "  <path id=\"{}\" class=\"edge {}\"{attrs} d=\"{d}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\"/>\n",
        escape(&edge.id),
        edge.kind.as_str(),
        style.stroke,
        num(style.width)
    ));
    let label = match (&style.keyword, &edge.label) {
        (Some(k), Some(l)) => Some(format!("{k} {l}")),
        (Some(k), None) => Some(k.clone()),
        (None, Some(l)) => Some(l.clone()),
        (None, None) => None,
    };
    if let Some(l) = label {
        let (mx, my) = midpoint(&pts);
        out.push_str(&text_el(mx, my - 4.0, "middle", 10.0, &format!(" fill=\"{}\"", style.stroke), &l));
    }
}

/// Depth-first node order: roots in declaration order, each followed by its
/// children, so a parent is always drawn beneath what it contains.
fn ordered<'a>(graph: &'a DiagramGraph) -> Vec<&'a Node> {
    fn walk<'a>(graph: &'a DiagramGraph, node: &'a Node, depth: usize, out: &mut Vec<&'a Node>) {
        out.push(node);
        if depth < graph.nodes.len() {
            for c in graph.children_of(&node.id) {
                walk(graph, c, depth + 1, out);
            }
        }
    }
    let mut out = Vec::with_capacity(graph.nodes.len());
    for n in graph.nodes.iter().filter(|n| n.parent.as_deref().map(|p| graph.node(p).is_none()).unwrap_or(true)) {
        walk(graph, n, 0, &mut out);
    }
    out
}

/// Draw a fully pinned graph as standalone SVG; `None` when any node lacks a
/// pin (or the graph is empty).
pub fn render_svg(graph: &DiagramGraph, links: &dyn Fn(&str) -> Option<String>) -> Option<String> {
    if !graph.is_fully_pinned() {
        return None;
    }
    let nodes = ordered(graph);
    // Absolute rectangles in pin space.
    let mut raw: HashMap<&str, AbsRect> = HashMap::new();
    for n in &nodes {
        let (x, y) = origin_of(graph, n);
        let (w, h) = size_of(graph, n, 0);
        raw.insert(n.id.as_str(), AbsRect { x, y, w, h });
    }
    let min_x = raw.values().map(|r| r.x).fold(f64::INFINITY, f64::min);
    let min_y = raw.values().map(|r| r.y).fold(f64::INFINITY, f64::min);
    let shift = (MARGIN - min_x, MARGIN - min_y);
    let rects: HashMap<&str, AbsRect> =
        raw.iter().map(|(k, r)| (*k, AbsRect { x: r.x + shift.0, y: r.y + shift.1, w: r.w, h: r.h })).collect();
    let width = rects.values().map(AbsRect::right).fold(0.0, f64::max) + MARGIN;
    let height = rects.values().map(AbsRect::bottom).fold(0.0, f64::max) + MARGIN;

    let mut body = String::new();
    for n in &nodes {
        let r = &rects[n.id.as_str()];
        // Decorations (a block's compartment, a free label) share their
        // owner's reference; the owner's group is the element's one anchor.
        let decoration = matches!(n.kind, NodeKind::Compartment | NodeKind::Label);
        match links(&n.element_ref).filter(|_| !decoration) {
            Some(url) => {
                let u = escape(&url);
                body.push_str(&format!("  <a xlink:href=\"{u}\" href=\"{u}\" target=\"_blank\" rel=\"noopener\">\n"));
                draw_node(n, r, &mut body);
                body.push_str("  </a>\n");
            }
            None => draw_node(n, r, &mut body),
        }
    }
    let mut used: Vec<&'static str> = Vec::new();
    for e in &graph.edges {
        draw_edge(e, &rects, shift, &mut used, &mut body);
    }

    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:sysml=\"urn:syscribe:1.0\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" viewBox=\"0 0 {} {}\" width=\"{}\" height=\"{}\" class=\"syscribe-diagram {}\" sysml:ref=\"{}\">\n",
        num(width),
        num(height),
        num(width),
        num(height),
        graph.kind.as_str(),
        escape(&graph.qualified_name)
    );
    out.push_str(&format!("  <title>{}</title>\n", escape(&graph.name)));
    if !used.is_empty() {
        out.push_str("  <defs>\n");
        for m in &used {
            out.push_str(&marker_def(m));
        }
        out.push_str("  </defs>\n");
    }
    out.push_str(&body);
    out.push_str("</svg>\n");
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vis::ir::{DiagramKind, EdgeKind, PortDirection, Rect};

    fn node(id: &str, kind: NodeKind, parent: Option<&str>, pin: Option<(f64, f64, f64, f64)>) -> Node {
        Node {
            id: id.into(),
            element_ref: format!("Sys::{id}"),
            resolved: true,
            element_type: None,
            kind,
            label: id.to_uppercase(),
            stereotype: None,
            parent: parent.map(str::to_string),
            direction: None,
            side: None,
            lines: vec![],
            is_abstract: false,
            pin: pin.map(|(x, y, w, h)| Rect { x, y, w: Some(w), h: Some(h) }),
            banners: vec![],
        }
    }

    fn no_links(_: &str) -> Option<String> {
        None
    }

    /// boundary `sys` (unsized) ⊃ block `engine` at (20,40) 160×50 with port
    /// `out` at (154,19) ⊃ block `motor` at (260,40) 160×50 with port `in`;
    /// one flow edge out → in.
    fn ibd() -> DiagramGraph {
        let mut g = DiagramGraph::empty(DiagramKind::Ibd, "Diagrams::D", "D", Some("Sys"));
        let mut sys = node("sys", NodeKind::Boundary, None, None);
        sys.pin = Some(Rect { x: 0.0, y: 0.0, w: None, h: None });
        sys.element_type = Some("PartDef".into());
        sys.stereotype = Some("part def".into());
        g.nodes.push(sys);
        let mut engine = node("engine", NodeKind::Block, Some("sys"), Some((20.0, 40.0, 160.0, 50.0)));
        engine.element_type = Some("Part".into());
        engine.is_abstract = true;
        g.nodes.push(engine);
        let mut pout = node("out", NodeKind::Port, Some("engine"), Some((154.0, 19.0, 12.0, 12.0)));
        pout.direction = Some(PortDirection::Out);
        g.nodes.push(pout);
        g.nodes.push(node("motor", NodeKind::Block, Some("sys"), Some((260.0, 40.0, 160.0, 50.0))));
        let mut pin = node("in", NodeKind::Port, Some("motor"), Some((-6.0, 19.0, 12.0, 12.0)));
        pin.direction = Some(PortDirection::In);
        g.nodes.push(pin);
        g.edges.push(Edge {
            id: "e-flow".into(),
            element_ref: Some("Sys::Link".into()),
            source: "out".into(),
            target: "in".into(),
            kind: EdgeKind::Flow,
            label: Some("power".into()),
            waypoints: None,
        });
        g
    }

    #[test]
    fn refuses_an_unpinned_graph() {
        let mut g = ibd();
        assert!(render_svg(&g, &no_links).is_some());
        g.nodes[3].pin = None;
        assert!(render_svg(&g, &no_links).is_none());
        assert!(render_svg(&DiagramGraph::empty(DiagramKind::Bdd, "D", "D", None), &no_links).is_none());
    }

    #[test]
    fn geometry_is_parent_relative_and_containers_bound_their_children() {
        let g = ibd();
        let s = render_svg(&g, &no_links).unwrap();
        assert!(s.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:sysml=\"urn:syscribe:1.0\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" viewBox=\"0 0 "), "{s}");
        // engine: origin (0,0)+(20,40), shifted by the 20 margin → (40,60).
        assert!(s.contains("<g id=\"engine\" class=\"block Part abstract\" sysml:ref=\"Sys::engine\">\n    <rect x=\"40\" y=\"60\" width=\"160\" height=\"50\""), "{s}");
        // The port sits at engine's origin + its relative pin.
        assert!(s.contains("<g id=\"out\" class=\"port out\" sysml:ref=\"Sys::out\">\n    <rect x=\"194\" y=\"79\" width=\"12\" height=\"12\" fill=\"#333\""), "{s}");
        // The boundary bounds its children: right = 260+160+16 = 436, bottom = 40+50+16 = 106.
        assert!(s.contains("<g id=\"sys\" class=\"boundary PartDef\" sysml:ref=\"Sys::sys\">\n    <rect x=\"20\" y=\"20\" width=\"436\" height=\"106\" rx=\"8\""), "{s}");
        assert!(s.contains("viewBox=\"0 0 476 146\""), "{s}");
        assert!(s.contains("font-style=\"italic\""), "abstract name is italic");
        assert!(s.contains("«part def»"), "stereotype drawn");
    }

    #[test]
    fn edges_carry_sysml_attributes_markers_and_labels() {
        let s = render_svg(&ibd(), &no_links).unwrap();
        assert!(s.contains("<path id=\"e-flow\" class=\"edge flow\" sysml:ref=\"Sys::Link\" sysml:source=\"out\" sysml:target=\"in\" marker-end=\"url(#arrow-filled)\" d=\"M 206,85 L 274,85\""), "{s}");
        assert!(s.contains("<marker id=\"arrow-filled\""));
        assert!(!s.contains("<marker id=\"arrow-open\""), "only markers in use are defined");
        assert!(s.contains(">power</text>"));
        // A composition puts its diamond at the source; a binding its `=` keyword.
        let mut g = ibd();
        g.edges[0].kind = EdgeKind::Composition;
        let s = render_svg(&g, &no_links).unwrap();
        assert!(s.contains("marker-start=\"url(#arrow-composition)\"") && !s.contains("marker-end"), "{s}");
        g.edges[0].kind = EdgeKind::Binding;
        g.edges[0].label = None;
        let s = render_svg(&g, &no_links).unwrap();
        assert!(s.contains("stroke-dasharray=\"4,4\"") && s.contains(">=</text>"), "{s}");
    }

    #[test]
    fn waypoints_are_honoured_as_absolute_routing() {
        let mut g = ibd();
        g.edges[0].waypoints = Some(vec![Point { x: 230.0, y: 10.0 }]);
        let s = render_svg(&g, &no_links).unwrap();
        // The ends leave their ports towards the waypoint (shifted by the margin to 250,30).
        assert!(s.contains("d=\"M 205.45,79 L 250,30 L 276.73,79\""), "{s}");
    }

    #[test]
    fn links_wrap_the_node_group_exactly_as_req_trs_link_002_says() {
        let links = |r: &str| (r == "Sys::engine").then(|| "https://h.test/a.md?x=1&y=\"2\"".to_string());
        let s = render_svg(&ibd(), &links).unwrap();
        let wrapper = "  <a xlink:href=\"https://h.test/a.md?x=1&amp;y=&quot;2&quot;\" href=\"https://h.test/a.md?x=1&amp;y=&quot;2&quot;\" target=\"_blank\" rel=\"noopener\">\n  <g id=\"engine\"";
        assert!(s.contains(wrapper), "{s}");
        assert_eq!(s.matches("<a ").count(), 1, "only the linked node is wrapped");
        assert!(s.contains("  </g>\n  </a>\n"));
        assert!(!render_svg(&ibd(), &no_links).unwrap().contains("<a "));
    }

    #[test]
    fn text_is_escaped_and_output_is_deterministic() {
        let mut g = ibd();
        g.nodes[1].label = "a <b> & \"c\"".into();
        let s = render_svg(&g, &no_links).unwrap();
        assert!(s.contains(">a &lt;b&gt; &amp; &quot;c&quot;</text>"), "{s}");
        assert_eq!(s, render_svg(&g, &no_links).unwrap());
    }

    #[test]
    fn negative_pins_are_shifted_into_view() {
        let mut g = DiagramGraph::empty(DiagramKind::Custom, "D", "D", None);
        g.nodes.push(node("a", NodeKind::Block, None, Some((-100.0, -50.0, 10.0, 10.0))));
        let s = render_svg(&g, &no_links).unwrap();
        assert!(s.contains("<rect x=\"20\" y=\"20\" width=\"10\" height=\"10\""), "{s}");
        assert!(s.contains("viewBox=\"0 0 50 50\""), "{s}");
    }
}
