//! IR → static SVG (`REQ-TRS-VIS-010`, `REQ-TRS-VIS-016`, spec §8.16.5).
//!
//! [`render_svg`] draws any [`DiagramGraph`] that has nodes. A fully pinned
//! graph is drawn from its pins — absolute rectangles computed by walking
//! the parents (pins are parent-relative, as the sprotty client writes
//! them), a node without a measured `w`/`h` getting its carried size from
//! [`super::size`] (a container without one bounds its children plus
//! padding), edges following their pinned waypoints or a straight line
//! clipped to the end shapes, labels placed as the browser's `fixed`-mode
//! postprocessor places them. Any other graph is laid out first by the
//! embedded ELK ([`super::layout`]) and drawn from that result: ELK's node
//! and port rectangles, its edge polylines (bend points included) as the
//! paths, and its label positions as given.
//!
//! The output follows §8.16.5: the `sysml:` namespace, a `<g id class
//! sysml:ref>` per node, a `<path id class sysml:ref sysml:source
//! sysml:target>` per edge, and the shared visual language of
//! [`super::style`] (`REQ-TRS-VIS-012`). When the `links` closure yields a
//! URL for a node's reference, that node's `<g>` is wrapped in
//! `<a xlink:href href target="_blank" rel="noopener">` exactly as
//! `REQ-TRS-LINK-002` specifies; otherwise it is left unwrapped. Output is
//! deterministic: nodes depth-first in declaration order, then edges.

use std::collections::BTreeMap;

use super::ir::{DiagramGraph, Edge, EdgeKind, Node, NodeKind, Point};
use super::layout::{self, Bounds, EdgeRoute, Layout, LayoutError};
use super::metrics::DIAGRAM_FAMILIES;
use super::shape;
use super::size::{carried_size, size_graph_default, LabelRole, Sizes};
use super::style::{edge_style, node_style, port_style, ArrowHead, NodeStyle, EDGE_STROKE};

/// Margin around the drawing, in user units.
const MARGIN: f64 = 20.0;
/// Padding inside a pinned container whose size is computed from its children.
const PADDING: f64 = 16.0;
/// Height reserved for a pinned container's header when it has no children.
const HEADER: f64 = 36.0;

#[derive(Debug, thiserror::Error)]
pub enum SvgError {
    /// The graph has no nodes: nothing to draw.
    #[error("the diagram has no shapes to draw")]
    Empty,
    #[error(transparent)]
    Layout(#[from] LayoutError),
}

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

fn is_container(node: &Node) -> bool {
    matches!(
        node.kind,
        NodeKind::Boundary | NodeKind::SystemBoundary | NodeKind::Swimlane | NodeKind::Fragment | NodeKind::State | NodeKind::Action
    )
}

// ── the pinned path ───────────────────────────────────────────────────────

/// The size of a pinned node: its pin's `w`/`h`, else — for a container or
/// block with pinned children — the box bounding those children (relative
/// coordinates) plus padding, else its carried size.
fn pinned_size(graph: &DiagramGraph, sizes: &Sizes, node: &Node, depth: usize) -> (f64, f64) {
    if let Some(pin) = node.pin {
        if let (Some(w), Some(h)) = (pin.w, pin.h) {
            return (w, h);
        }
    }
    let carried = carried_size(node, sizes);
    let (dw, dh) = (carried.w, carried.h);
    if (is_container(node) || node.kind == NodeKind::Block) && depth < graph.nodes.len() {
        let mut right: f64 = 0.0;
        let mut bottom: f64 = 0.0;
        let mut any = false;
        for child in graph.children_of(&node.id).filter(|c| matches!(c.kind, NodeKind::Compartment | NodeKind::Label) == false) {
            let Some(p) = child.pin else { continue };
            let (cw, ch) = pinned_size(graph, sizes, child, depth + 1);
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

/// Which side of its parent a port sits on, from its centre.
fn side_of(port: &Bounds, parent: &Bounds) -> &'static str {
    let (cx, cy) = (port.cx() - parent.x, port.cy() - parent.y);
    let mut dist = [("west", cx.abs()), ("east", (cx - parent.w).abs()), ("north", cy.abs()), ("south", (cy - parent.h).abs())];
    dist.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    dist[0].0
}

/// The point halfway along a polyline.
fn midpoint(pts: &[Point]) -> Point {
    let len = |a: &Point, b: &Point| ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt();
    let total: f64 = pts.windows(2).map(|w| len(&w[0], &w[1])).sum();
    let mut remaining = total / 2.0;
    for w in pts.windows(2) {
        let l = len(&w[0], &w[1]);
        if l >= remaining || l == 0.0 {
            let f = if l == 0.0 { 0.0 } else { remaining / l };
            return Point { x: w[0].x + (w[1].x - w[0].x) * f, y: w[0].y + (w[1].y - w[0].y) * f };
        }
        remaining -= l;
    }
    pts.last().copied().unwrap_or(Point { x: 0.0, y: 0.0 })
}

/// Geometry of a fully pinned graph without a layout engine (module doc).
/// Public so a caller can draw pins exactly as `render_svg` does.
pub fn pinned_layout(graph: &DiagramGraph, sizes: &Sizes) -> Layout {
    let mut out = Layout { algorithm: "pinned".to_string(), nodes: BTreeMap::new(), labels: BTreeMap::new(), edges: BTreeMap::new(), width: 0.0, height: 0.0 };
    for n in &graph.nodes {
        let (x, y) = origin_of(graph, n);
        let (w, h) = pinned_size(graph, sizes, n, 0);
        out.nodes.insert(n.id.clone(), Bounds { x, y, w, h });
    }
    // Labels, as the client's `placeLabelsFixed` and `vbox` place them.
    for n in &graph.nodes {
        let Some(sizing) = sizes.node(&n.id) else { continue };
        let b = out.nodes[&n.id];
        match n.kind {
            NodeKind::Port => {
                let parent = n.parent.as_deref().and_then(|p| out.nodes.get(p)).copied().unwrap_or(b);
                for l in &sizing.labels {
                    let (lx, ly) = match side_of(&b, &parent) {
                        "west" => (-l.w - 1.0, b.h + 1.0),
                        "east" => (b.w + 1.0, b.h + 1.0),
                        "north" => ((b.w - l.w) / 2.0, -l.h - 2.0),
                        _ => ((b.w - l.w) / 2.0, b.h + 2.0),
                    };
                    out.labels.insert(l.id.clone(), Bounds { x: b.x + lx, y: b.y + ly, w: l.w, h: l.h });
                }
            }
            NodeKind::Compartment | NodeKind::Label => {
                for l in &sizing.labels {
                    out.labels.insert(l.id.clone(), Bounds { x: b.x + l.x, y: b.y + l.y, w: l.w, h: l.h });
                }
            }
            _ => {
                let compound = graph.children_of(&n.id).any(|c| !matches!(c.kind, NodeKind::Port | NodeKind::Compartment | NodeKind::Label));
                // A fragment's labels sit in its top-left keyword tab even
                // when it nests nothing (`REQ-TRS-VIS-021`).
                let top_left = compound || n.kind == NodeKind::Fragment;
                let mut y = 4.0;
                if super::shape::is_symbol(n.kind) {
                    // Centred in the symbol, as ELK places them (`V_CENTER`).
                    let stack: f64 = sizing.labels.iter().filter(|l| l.role != LabelRole::Free).map(|l| l.h + 1.0).sum::<f64>() - 1.0;
                    y = ((b.h - stack) / 2.0).max(4.0);
                }
                for l in sizing.labels.iter().filter(|l| l.role != LabelRole::Free) {
                    let x = if top_left { 8.0 } else { ((b.w - l.w) / 2.0).max(0.0) };
                    out.labels.insert(l.id.clone(), Bounds { x: b.x + x, y: b.y + y, w: l.w, h: l.h });
                    y += l.h + 1.0;
                }
            }
        }
    }
    for e in &graph.edges {
        let (Some(s), Some(t)) = (out.nodes.get(&e.source), out.nodes.get(&e.target)) else { continue };
        if matches!(e.kind, EdgeKind::Message | EdgeKind::Return) {
            // A sequence message with pinned waypoints runs along them alone
            // (stem to stem at its row, `REQ-TRS-VIS-021`): the header boxes
            // are not its ends, so there is nothing to clip. Labels stack
            // above the midpoint, as for every pinned edge.
            if let Some(wp) = e.waypoints.as_ref().filter(|w| w.len() >= 2) {
                let m = midpoint(wp);
                if let Some(labels) = sizes.edges.get(&e.id) {
                    let stack: f64 = labels.iter().map(|l| l.h + 1.0).sum();
                    let mut y = m.y - stack - 3.0;
                    for l in labels {
                        out.labels.insert(l.id.clone(), Bounds { x: m.x - l.w / 2.0, y, w: l.w, h: l.h });
                        y += l.h + 1.0;
                    }
                }
                out.edges.insert(e.id.clone(), EdgeRoute { points: wp.clone(), routed: true });
                continue;
            }
        }
        let mut points = vec![Point { x: s.cx(), y: s.cy() }];
        points.extend(e.waypoints.iter().flatten().copied());
        points.push(Point { x: t.cx(), y: t.cy() });
        let m = midpoint(&clipped(&points, s, t));
        if let Some(labels) = sizes.edges.get(&e.id) {
            let stack: f64 = labels.iter().map(|l| l.h + 1.0).sum();
            let mut y = m.y - stack - 3.0;
            for l in labels {
                out.labels.insert(l.id.clone(), Bounds { x: m.x - l.w / 2.0, y, w: l.w, h: l.h });
                y += l.h + 1.0;
            }
        }
        out.edges.insert(e.id.clone(), EdgeRoute { points, routed: false });
    }
    out
}

/// Where a straight line from `from`'s centre towards `towards` leaves
/// `from`'s border.
fn clip(from: &Bounds, towards: Point) -> Point {
    let (cx, cy) = (from.cx(), from.cy());
    let (dx, dy) = (towards.x - cx, towards.y - cy);
    if dx.abs() < f64::EPSILON && dy.abs() < f64::EPSILON {
        return Point { x: cx, y: cy };
    }
    let tx = if dx.abs() < f64::EPSILON { f64::INFINITY } else { (from.w / 2.0) / dx.abs() };
    let ty = if dy.abs() < f64::EPSILON { f64::INFINITY } else { (from.h / 2.0) / dy.abs() };
    let t = tx.min(ty);
    Point { x: cx + dx * t, y: cy + dy * t }
}

/// An unrouted polyline (centre to centre, with any waypoints between) with
/// its two ends clipped to the end shapes' borders.
fn clipped(points: &[Point], s: &Bounds, t: &Bounds) -> Vec<Point> {
    let n = points.len();
    if n < 2 {
        return points.to_vec();
    }
    let mut pts = Vec::with_capacity(n);
    pts.push(clip(s, points[1]));
    pts.extend_from_slice(&points[1..n - 1]);
    pts.push(clip(t, points[n - 2]));
    pts
}

// ── drawing ───────────────────────────────────────────────────────────────

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

fn font_family() -> String {
    let mut f = DIAGRAM_FAMILIES.join(", ");
    f.push_str(", sans-serif");
    f
}

fn text_el(x: f64, y: f64, anchor: &str, size: f64, extra: &str, s: &str) -> String {
    format!(
        "    <text x=\"{}\" y=\"{}\" text-anchor=\"{anchor}\" font-size=\"{}\" font-family=\"{}\"{extra}>{}</text>\n",
        num(x),
        num(y),
        num(size),
        font_family(),
        escape(s)
    )
}

/// Draw one label box: centred in its box, except compartment lines, which
/// start at the box's left edge. The baseline sits one font size below the
/// box's top.
fn label_el(b: &Bounds, text: &str, role: LabelRole, fill: &str, italic: bool, bold: bool, halo: bool) -> String {
    let (font, _, role_italic) = role.font();
    let mut extra = String::new();
    if bold {
        extra.push_str(" font-weight=\"bold\"");
    }
    if italic || role_italic {
        extra.push_str(" font-style=\"italic\"");
    }
    extra.push_str(&format!(" fill=\"{fill}\""));
    if halo {
        extra.push_str(" stroke=\"#fff\" stroke-width=\"3\" paint-order=\"stroke\"");
    }
    match role {
        LabelRole::Line => text_el(b.x, b.y + font, "start", font, &extra, text),
        _ => text_el(b.cx(), b.y + font, "middle", font, &extra, text),
    }
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

struct Drawer<'a> {
    sizes: &'a Sizes,
    layout: &'a Layout,
    shift: (f64, f64),
}

impl Drawer<'_> {
    fn rect(&self, id: &str) -> Option<Bounds> {
        self.layout.nodes.get(id).map(|b| Bounds { x: b.x + self.shift.0, y: b.y + self.shift.1, w: b.w, h: b.h })
    }

    fn label(&self, id: &str) -> Option<Bounds> {
        self.layout.labels.get(id).map(|b| Bounds { x: b.x + self.shift.0, y: b.y + self.shift.1, w: b.w, h: b.h })
    }

    /// Where a lifeline's stem ends: just below the lowest node of the
    /// drawing (sequence diagrams, `REQ-TRS-VIS-021`).
    fn stem_bottom(&self) -> f64 {
        self.layout.nodes.values().map(Bounds::bottom).fold(0.0, f64::max) + self.shift.1 + 12.0
    }

    /// The labels of `node` (name, stereotype, banners, lines, port name)
    /// with their boxes.
    fn labels_of(&self, node: &Node, style: &NodeStyle, out: &mut String) {
        let Some(sizing) = self.sizes.node(&node.id) else { return };
        for l in sizing.labels.iter().filter(|l| l.role != LabelRole::Free) {
            let Some(b) = self.label(&l.id) else { continue };
            let (fill, bold, italic) = match l.role {
                LabelRole::Name => (style.text.as_str(), true, node.is_abstract),
                LabelRole::Stereotype | LabelRole::Banner => (style.stroke.as_str(), false, false),
                LabelRole::Line | LabelRole::Value => ("#333", false, false),
                LabelRole::Status => (style.stroke.as_str(), true, false),
                LabelRole::Badge => (style.stroke.as_str(), false, false),
                LabelRole::PortName => (style.text.as_str(), false, false),
                _ => (style.text.as_str(), false, false),
            };
            out.push_str(&label_el(&b, &l.text, l.role, fill, italic, bold, false));
        }
    }

    fn draw_node(&self, node: &Node, r: &Bounds, out: &mut String) {
        out.push_str(&format!(
            "  <g id=\"{}\" class=\"{}\" sysml:ref=\"{}\">\n",
            escape(&node.id),
            escape(&node_class(node)),
            escape(&node.element_ref)
        ));
        let style = node_style(node);
        let dash = if style.dashed { " stroke-dasharray=\"4,3\"" } else { "" };
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
                self.labels_of(node, &style, out);
            }
            NodeKind::Label => {
                let (font, _, _) = LabelRole::Free.font();
                out.push_str(&text_el(r.x, r.y + font, "start", font, &format!(" fill=\"{}\"", style.text), &node.label));
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
            NodeKind::Fork | NodeKind::Join => {
                // A thick synchronisation bar (REQ-TRS-VIS-019); its name sits beside it.
                out.push_str(&format!(
                    "    <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"2\" fill=\"{}\" stroke=\"none\"/>\n",
                    num(r.x),
                    num(r.y),
                    num(r.w),
                    num(r.h),
                    style.fill
                ));
                self.labels_of(node, &style, out);
            }
            NodeKind::Decision | NodeKind::Merge => {
                // A diamond; a decision's condition is drawn beside it.
                let (cx, cy) = (r.cx(), r.cy());
                out.push_str(&format!(
                    "    <path d=\"M {},{} L {},{} L {},{} L {},{} z\" fill=\"{}\" stroke=\"{}\" stroke-width=\"1.4\"{dash}/>\n",
                    num(cx),
                    num(r.y),
                    num(r.right()),
                    num(cy),
                    num(cx),
                    num(r.bottom()),
                    num(r.x),
                    num(cy),
                    style.fill,
                    style.stroke
                ));
                self.labels_of(node, &style, out);
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
                self.labels_of(node, &style, out);
            }
            // Sequence kinds (spec §8.16.8.3, `REQ-TRS-VIS-021`).
            NodeKind::Lifeline | NodeKind::Actor => {
                // A header box (a lifeline) or a stick figure under the name
                // (an actor), then a dashed stem down to the drawing's bottom.
                let cx = r.cx();
                if node.kind == NodeKind::Lifeline {
                    out.push_str(&format!(
                        "    <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"1.4\"{dash}/>\n",
                        num(r.x),
                        num(r.y),
                        num(r.w),
                        num(r.h),
                        style.fill,
                        style.stroke
                    ));
                } else {
                    let top = self.label(&format!("{}-label", node.id)).map(|b| b.bottom() + 2.0).unwrap_or(r.y + 20.0);
                    let (head_r, body, arms, legs) = (5.0, 12.0, 9.0, 10.0);
                    let (hy, by, ly) = (top + head_r, top + 2.0 * head_r, top + 2.0 * head_r + body);
                    out.push_str(&format!(
                        "    <circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"1.4\"{dash}/>\n",
                        num(cx), num(hy), num(head_r), style.fill, style.stroke
                    ));
                    out.push_str(&format!(
                        "    <path d=\"M {cx},{by} L {cx},{ly} M {ax},{ay} L {bx},{ay} M {cx},{ly} L {lx1},{ly2} M {cx},{ly} L {lx2},{ly2}\" fill=\"none\" stroke=\"{}\" stroke-width=\"1.4\"{dash}/>\n",
                        style.stroke,
                        cx = num(cx),
                        by = num(by),
                        ly = num(ly),
                        ax = num(cx - arms),
                        bx = num(cx + arms),
                        ay = num(by + 4.0),
                        lx1 = num(cx - 7.0),
                        lx2 = num(cx + 7.0),
                        ly2 = num(ly + legs)
                    ));
                }
                let bottom = self.stem_bottom();
                if bottom > r.bottom() {
                    out.push_str(&format!(
                        "    <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"1.2\" stroke-dasharray=\"6,4\"/>\n",
                        num(cx),
                        num(r.bottom()),
                        num(cx),
                        num(bottom),
                        style.stroke
                    ));
                }
                self.labels_of(node, &style, out);
            }
            NodeKind::Activation => {
                // A narrow bar on the stem; it carries no text.
                out.push_str(&format!(
                    "    <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"1.2\"/>\n",
                    num(r.x),
                    num(r.y),
                    num(r.w),
                    num(r.h),
                    style.fill,
                    style.stroke
                ));
            }
            NodeKind::Fragment => {
                // An open box (the stems and messages show through) with a
                // keyword tab around its label stack in the top-left corner.
                out.push_str(&format!(
                    "    <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"1.4\"{dash}/>\n",
                    num(r.x),
                    num(r.y),
                    num(r.w),
                    num(r.h),
                    style.stroke
                ));
                if let Some(sizing) = self.sizes.node(&node.id) {
                    let boxes: Vec<Bounds> = sizing.labels.iter().filter(|l| l.role != LabelRole::Free).filter_map(|l| self.label(&l.id)).collect();
                    if let Some(bottom) = boxes.iter().map(Bounds::bottom).fold(None, |m: Option<f64>, b| Some(m.map_or(b, |m| m.max(b)))) {
                        let right = boxes.iter().map(Bounds::right).fold(r.x, f64::max) + 8.0;
                        let (tw, th) = ((right - r.x).min(r.w), (bottom + 4.0 - r.y).min(r.h));
                        out.push_str(&format!(
                            "    <path d=\"M {},{} L {},{} L {},{} L {},{} L {},{} z\" fill=\"{}\" stroke=\"{}\" stroke-width=\"1.2\"/>\n",
                            num(r.x),
                            num(r.y),
                            num(r.x + tw),
                            num(r.y),
                            num(r.x + tw),
                            num(r.y + th - 6.0),
                            num(r.x + tw - 6.0),
                            num(r.y + th),
                            num(r.x),
                            num(r.y + th),
                            style.fill,
                            style.stroke
                        ));
                    }
                }
                self.labels_of(node, &style, out);
            }
            _ if shape::shape_of(node.kind, r.x, r.y, r.w, r.h).is_some() => {
                // A safety-diagram symbol (GH #223): the outline from `vis::shape`.
                let sh = shape::shape_of(node.kind, r.x, r.y, r.w, r.h).unwrap_or(shape::Shape { outline: shape::Outline::Rect { rx: 0.0 }, extras: vec![] });
                let sw = style.stroke_width.unwrap_or(1.4);
                let paint = format!("fill=\"{}\" stroke=\"{}\" stroke-width=\"{}\"{dash}", style.fill, style.stroke, num(sw));
                match &sh.outline {
                    shape::Outline::Rect { rx } => out.push_str(&format!(
                        "    <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" {paint}/>\n",
                        num(r.x), num(r.y), num(r.w), num(r.h), num(*rx)
                    )),
                    shape::Outline::Ellipse => out.push_str(&format!(
                        "    <ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" {paint}/>\n",
                        num(r.cx()), num(r.cy()), num(r.w / 2.0), num(r.h / 2.0)
                    )),
                    shape::Outline::Path(d) => out.push_str(&format!("    <path d=\"{d}\" {paint}/>\n")),
                }
                for extra in &sh.extras {
                    match extra {
                        shape::Extra::Stroke(d) => out.push_str(&format!("    <path d=\"{d}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\"/>\n", style.stroke, num(sw))),
                        shape::Extra::Circle { cx, cy, r } => out.push_str(&format!(
                            "    <circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"#fff\" stroke=\"{}\" stroke-width=\"{}\"/>\n",
                            num(*cx), num(*cy), num(*r), style.stroke, num(sw)
                        )),
                        shape::Extra::Diamond(d) => out.push_str(&format!("    <path d=\"{d}\" fill=\"#fff\" stroke=\"{}\" stroke-width=\"{}\"/>\n", style.stroke, num(sw))),
                        shape::Extra::Letter { x, y, text } => out.push_str(&text_el(*x, *y, "start", 10.0, &format!(" fill=\"{}\" font-weight=\"bold\"", style.stroke), text)),
                    }
                }
                self.labels_of(node, &style, out);
            }
            _ => {
                let rounded = matches!(node.kind, NodeKind::Boundary | NodeKind::State | NodeKind::Action | NodeKind::UseCase | NodeKind::SystemBoundary);
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
                if let Some(hf) = &style.header_fill {
                    // The header band ends under the name label.
                    let header_h = self
                        .label(&format!("{}-label", node.id))
                        .map(|b| b.bottom() - r.y + 4.0)
                        .unwrap_or(HEADER)
                        .min(r.h);
                    out.push_str(&format!(
                        "    <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"{rx} fill=\"{hf}\" fill-opacity=\"0.15\" stroke=\"none\"/>\n",
                        num(r.x),
                        num(r.y),
                        num(r.w),
                        num(header_h)
                    ));
                }
                self.labels_of(node, &style, out);
            }
        }
        out.push_str("  </g>\n");
    }

    /// A feature diagram's notation (`REQ-TRS-FMED-001`): a filled circle above a
    /// mandatory feature and a hollow one above an optional one, and under a feature
    /// with an alternative or or group a wedge across its children (hollow for
    /// alternative, filled for or).
    fn draw_feature_marks(&self, graph: &DiagramGraph, out: &mut String) {
        for n in graph.nodes.iter().filter(|n| n.kind == NodeKind::Feature) {
            let (Some(mark), Some(r)) = (n.feature.as_ref(), self.rect(&n.id)) else { continue };
            let has_parent = graph.edges.iter().any(|e| e.kind == EdgeKind::FeatureChild && e.target == n.id);
            if has_parent {
                out.push_str(&format!(
                    "  <circle class=\"feature-mark\" cx=\"{}\" cy=\"{}\" r=\"5\" fill=\"{}\" stroke=\"#2b3440\" stroke-width=\"1.5\"/>\n",
                    num(r.cx()),
                    num(r.y - 7.0),
                    if mark.mandatory { "#2b3440" } else { "#fff" }
                ));
            }
            if mark.group != "alternative" && mark.group != "or" {
                continue;
            }
            let angles: Vec<f64> = graph
                .edges
                .iter()
                .filter(|e| e.kind == EdgeKind::FeatureChild && e.source == n.id)
                .filter_map(|e| self.rect(&e.target))
                .map(|c| (c.y - r.bottom()).max(1.0).atan2(c.cx() - r.cx()))
                .collect();
            if angles.len() < 2 {
                continue;
            }
            let lo = angles.iter().cloned().fold(f64::INFINITY, f64::min);
            let hi = angles.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let rad = 24.0;
            let at = |a: f64| format!("{},{}", num(r.cx() + rad * a.cos()), num(r.bottom() + rad * a.sin()));
            out.push_str(&format!(
                "  <path class=\"group-arc\" d=\"M {},{} L {} A {rad} {rad} 0 {} 1 {} Z\" fill=\"{}\" stroke=\"#44546a\" stroke-width=\"1.2\"/>\n",
                num(r.cx()),
                num(r.bottom()),
                at(lo),
                if hi - lo > std::f64::consts::PI { 1 } else { 0 },
                at(hi),
                if mark.group == "or" { "#44546a" } else { "none" }
            ));
        }
    }

    fn draw_edge(&self, edge: &Edge, used: &mut Vec<&'static str>, out: &mut String) {
        let Some(route) = self.layout.edges.get(&edge.id) else { return };
        let (Some(s), Some(t)) = (self.rect(&edge.source), self.rect(&edge.target)) else { return };
        let style = edge_style(edge.kind);
        let shifted: Vec<Point> = route.points.iter().map(|p| Point { x: p.x + self.shift.0, y: p.y + self.shift.1 }).collect();
        let pts = if route.routed { shifted } else { clipped(&shifted, &s, &t) };
        if pts.len() < 2 {
            return;
        }
        let d = pts
            .iter()
            .enumerate()
            .map(|(i, p)| format!("{} {},{}", if i == 0 { "M" } else { "L" }, num(p.x), num(p.y)))
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
        if let Some(labels) = self.sizes.edges.get(&edge.id) {
            for l in labels {
                let Some(b) = self.label(&l.id) else { continue };
                out.push_str(&label_el(&b, &l.text, l.role, &style.stroke, false, false, true));
            }
        }
    }
}

/// Depth-first node order: roots in declaration order, each followed by its
/// children, so a parent is always drawn beneath what it contains.
fn ordered(graph: &DiagramGraph) -> Vec<&Node> {
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

/// Draw `graph` as standalone SVG with the process-wide diagram metrics.
pub fn render_svg(graph: &DiagramGraph, links: &dyn Fn(&str) -> Option<String>) -> Result<String, SvgError> {
    render_svg_with(graph, &size_graph_default(graph), links)
}

/// [`render_svg`] with precomputed sizes: from pins when every node has one,
/// else from the embedded ELK.
pub fn render_svg_with(graph: &DiagramGraph, sizes: &Sizes, links: &dyn Fn(&str) -> Option<String>) -> Result<String, SvgError> {
    if graph.nodes.is_empty() {
        return Err(SvgError::Empty);
    }
    let laid = if graph.is_fully_pinned() { pinned_layout(graph, sizes) } else { layout::layout(graph, sizes)? };
    Ok(draw(graph, sizes, &laid, links))
}

/// Draw `graph` from a [`Layout`] (from [`pinned_layout`] or
/// [`super::layout::layout`]).
pub fn draw(graph: &DiagramGraph, sizes: &Sizes, laid: &Layout, links: &dyn Fn(&str) -> Option<String>) -> String {
    let nodes = ordered(graph);
    let boxes = laid.nodes.values().chain(laid.labels.values());
    let min_x = boxes.clone().map(|b| b.x).fold(f64::INFINITY, f64::min);
    let min_y = boxes.clone().map(|b| b.y).fold(f64::INFINITY, f64::min);
    let shift = (MARGIN - min_x, MARGIN - min_y);
    let width = boxes.clone().map(Bounds::right).fold(0.0, f64::max) + shift.0 + MARGIN;
    let height = boxes.map(Bounds::bottom).fold(0.0, f64::max) + shift.1 + MARGIN;
    let d = Drawer { sizes, layout: laid, shift };

    let mut body = String::new();
    for n in &nodes {
        let Some(r) = d.rect(&n.id) else { continue };
        // Decorations (a block's compartment, a free label) share their
        // owner's reference; the owner's group is the element's one anchor.
        let decoration = matches!(n.kind, NodeKind::Compartment | NodeKind::Label);
        match links(&n.element_ref).filter(|_| !decoration) {
            Some(url) => {
                let u = escape(&url);
                body.push_str(&format!("  <a xlink:href=\"{u}\" href=\"{u}\" target=\"_blank\" rel=\"noopener\">\n"));
                d.draw_node(n, &r, &mut body);
                body.push_str("  </a>\n");
            }
            None => d.draw_node(n, &r, &mut body),
        }
    }
    let mut used: Vec<&'static str> = Vec::new();
    for e in &graph.edges {
        d.draw_edge(e, &mut used, &mut body);
    }
    d.draw_feature_marks(graph, &mut body);

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
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vis::ir::{DiagramKind, EdgeKind, PortDirection, Rect};
    use crate::vis::metrics::ApproxMetrics;
    use crate::vis::size::size_graph;

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
            feature: None,
            mark: None,
        }
    }

    fn no_links(_: &str) -> Option<String> {
        None
    }

    /// Render with the approximate metrics so the numbers do not depend on
    /// the fonts installed.
    fn svg(g: &DiagramGraph, links: &dyn Fn(&str) -> Option<String>) -> Result<String, SvgError> {
        render_svg_with(g, &size_graph(g, &ApproxMetrics), links)
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
    fn an_empty_graph_is_refused_and_an_unpinned_one_is_laid_out() {
        let mut g = ibd();
        assert!(svg(&g, &no_links).is_ok());
        assert!(matches!(svg(&DiagramGraph::empty(DiagramKind::Bdd, "D", "D", None), &no_links), Err(SvgError::Empty)));
        // One unpinned node: ELK lays the whole graph out.
        g.nodes[3].pin = None;
        let s = svg(&g, &no_links).expect("laid out by ELK");
        assert!(s.contains("<g id=\"motor\""), "{s}");
        assert!(s.contains("<path id=\"e-flow\" class=\"edge flow\""), "{s}");
        // No pins at all.
        for n in &mut g.nodes {
            n.pin = None;
        }
        let s = svg(&g, &no_links).unwrap();
        assert_eq!(s.matches("<g id=\"").count(), 5);
        assert!(s.contains("sysml:source=\"out\" sysml:target=\"in\""));
    }

    #[test]
    fn geometry_is_parent_relative_and_containers_bound_their_children() {
        let g = ibd();
        let s = svg(&g, &no_links).unwrap();
        assert!(s.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:sysml=\"urn:syscribe:1.0\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" viewBox=\"0 0 "), "{s}");
        // engine: origin (0,0)+(20,40), shifted by the 20 margin → (40,60).
        assert!(s.contains("<g id=\"engine\" class=\"block Part abstract\" sysml:ref=\"Sys::engine\">\n    <rect x=\"40\" y=\"60\" width=\"160\" height=\"50\""), "{s}");
        // The port sits at engine's origin + its relative pin.
        assert!(s.contains("<g id=\"out\" class=\"port out\" sysml:ref=\"Sys::out\">\n    <rect x=\"194\" y=\"79\" width=\"12\" height=\"12\" fill=\"#333\""), "{s}");
        // The boundary bounds its children: right = 260+160+16 = 436, bottom = 40+50+16 = 106.
        assert!(s.contains("<g id=\"sys\" class=\"boundary PartDef\" sysml:ref=\"Sys::sys\">\n    <rect x=\"20\" y=\"20\" width=\"436\" height=\"106\" rx=\"8\""), "{s}");
        assert!(s.contains("font-style=\"italic\""), "abstract name is italic");
        assert!(s.contains("«part def»"), "stereotype drawn");
        // The container's labels sit top-left, a block's centred.
        assert!(s.contains("text-anchor=\"middle\"") && s.contains(">ENGINE</text>"));
    }

    #[test]
    fn edges_carry_sysml_attributes_markers_and_labels() {
        let s = svg(&ibd(), &no_links).unwrap();
        assert!(s.contains("<path id=\"e-flow\" class=\"edge flow\" sysml:ref=\"Sys::Link\" sysml:source=\"out\" sysml:target=\"in\" marker-end=\"url(#arrow-filled)\" d=\"M 206,85 L 274,85\""), "{s}");
        assert!(s.contains("<marker id=\"arrow-filled\""));
        assert!(!s.contains("<marker id=\"arrow-open\""), "only markers in use are defined");
        assert!(s.contains(">power</text>"));
        // A composition puts its diamond at the source; a binding its `=` keyword.
        let mut g = ibd();
        g.edges[0].kind = EdgeKind::Composition;
        let s = svg(&g, &no_links).unwrap();
        assert!(s.contains("marker-start=\"url(#arrow-composition)\"") && !s.contains("marker-end"), "{s}");
        g.edges[0].kind = EdgeKind::Binding;
        g.edges[0].label = None;
        let s = svg(&g, &no_links).unwrap();
        assert!(s.contains("stroke-dasharray=\"4,4\"") && s.contains(">=</text>"), "{s}");
    }

    #[test]
    fn waypoints_are_honoured_as_absolute_routing() {
        let mut g = ibd();
        g.edges[0].waypoints = Some(vec![Point { x: 230.0, y: 10.0 }]);
        // No label: a label stacked above the waypoint would widen the drawing's margin.
        g.edges[0].label = None;
        let s = svg(&g, &no_links).unwrap();
        // The ends leave their ports towards the waypoint (shifted by the margin to 250,30).
        assert!(s.contains("d=\"M 205.45,79 L 250,30 L 276.73,79\""), "{s}");
    }

    #[test]
    fn links_wrap_the_node_group_exactly_as_req_trs_link_002_says() {
        let links = |r: &str| (r == "Sys::engine").then(|| "https://h.test/a.md?x=1&y=\"2\"".to_string());
        let s = svg(&ibd(), &links).unwrap();
        let wrapper = "  <a xlink:href=\"https://h.test/a.md?x=1&amp;y=&quot;2&quot;\" href=\"https://h.test/a.md?x=1&amp;y=&quot;2&quot;\" target=\"_blank\" rel=\"noopener\">\n  <g id=\"engine\"";
        assert!(s.contains(wrapper), "{s}");
        assert_eq!(s.matches("<a ").count(), 1, "only the linked node is wrapped");
        assert!(s.contains("  </g>\n  </a>\n"));
        assert!(!svg(&ibd(), &no_links).unwrap().contains("<a "));
    }

    #[test]
    fn text_is_escaped_and_output_is_deterministic() {
        let mut g = ibd();
        g.nodes[1].label = "a <b> & \"c\"".into();
        let s = svg(&g, &no_links).unwrap();
        assert!(s.contains(">a &lt;b&gt; &amp; &quot;c&quot;</text>"), "{s}");
        assert_eq!(s, svg(&g, &no_links).unwrap());
    }

    #[test]
    fn negative_pins_are_shifted_into_view() {
        let mut g = DiagramGraph::empty(DiagramKind::Custom, "D", "D", None);
        g.nodes.push(node("a", NodeKind::Block, None, Some((-100.0, -50.0, 10.0, 10.0))));
        let s = svg(&g, &no_links).unwrap();
        // The name label (centred, wider than the 10×10 pin) sets the left edge.
        assert!(s.contains("<rect x=\""), "{s}");
        assert!(s.contains("viewBox=\"0 0 "), "{s}");
        let laid = pinned_layout(&g, &size_graph(&g, &ApproxMetrics));
        assert_eq!(laid.nodes["a"], Bounds { x: -100.0, y: -50.0, w: 10.0, h: 10.0 });
    }
}
