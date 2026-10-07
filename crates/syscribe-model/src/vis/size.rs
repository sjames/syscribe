//! Rust-owned node sizing (`REQ-TRS-VIS-017`): every node, port, compartment
//! and label of a [`DiagramGraph`] gets a `(w, h)` from the shared text
//! metrics ([`super::metrics`]), computed exactly as the browser client's
//! micro-layout would have measured it (`frontend/src/layout.ts`
//! `prepareForLayout` + sprotty's `vbox`), so the sprotty graph can carry the
//! sizes and the browser's ELK and the embedded ELK ([`super::layout`]) start
//! from the same numbers.
//!
//! ## The client's recipe, mirrored
//!
//! - A **node** (`block`, `boundary`, `state`, …) is a `vbox` with
//!   `paddingTop/Bottom 4`, `paddingLeft/Right 0`, `vGap 1`, `hAlign left`.
//!   Its layoutable children, in order: the `«stereotype»` label, one
//!   `«banner»` label per applied stereotype, the name label, then the IR
//!   children that are compartments or free labels, in declaration order.
//!   Ports and nested nodes are not layoutable children (sprotty's `SPort` and
//!   `SNode` lack `layoutableChildFeature`), so they contribute nothing to
//!   the box. Width = widest child, height = children stacked with the gaps,
//!   plus the paddings.
//! - A **compartment** is a `vbox` with paddings `4/4/8/8` and `vGap 2` whose
//!   children are its `lines`, one label each.
//! - A **port** is a fixed 12×12 square with one name label.
//! - **Fonts** (`views.tsx` `SysmlLabelView`): name 12px bold, stereotype and
//!   banner 9px italic, compartment line 10px, port name 9px, edge keyword
//!   and label 10px, free label 12px.
//!
//! ## Safety margin
//!
//! A label box is `ceil(advance × 1.08 + 2)` wide and `ceil(font × 1.25)`
//! tall: 8 % plus two pixels over the measured advance absorbs the difference
//! between the metrics' font and whatever the browser substitutes, and the
//! 1.25 line factor is the usual ascent+descent of a sans face. The margin is
//! deliberately one-sided — a box is never smaller than the text. A leaf
//! node's width additionally reserves ELK's horizontal `nodeLabels.padding`
//! (8 a side), inside which ELK centres the label stack.
//!
//! ## Leaf nodes are exactly their carried size
//!
//! The client treats a server size as authoritative: a *leaf* node (one
//! with no nested nodes) is laid out by ELK with `PORTS MINIMUM_SIZE` and the
//! carried size as its minimum, so ELK never grows it for its labels and
//! only ever adds room for ports that do not fit on a side. A leaf's box is
//! therefore raised here to the client's minimum ([`MIN_NODE_WIDTH`] ×
//! [`MIN_NODE_HEIGHT`]) and to what its ports need on the sides they use
//! (`n` ports of 12 with ELK's 16 port spacing before, between and after:
//! `28n + 16`), so the browser and the executable both start from a box ELK
//! leaves alone. A *compound* node's box is only its label stack — ELK
//! sizes it around its children, with `max(120, w) × max(40, h)` as the
//! minimum.

use std::collections::BTreeMap;

use super::ir::{DiagramGraph, Edge, Node, NodeKind};
use super::metrics::TextMetrics;
use super::sprotty::{port_side, sprotty_type, TYPE_NODE};
use super::style::edge_style;

/// Port square side, as in the client (`PORT_SIZE`).
pub const PORT_SIZE: f64 = 12.0;
/// ELK's `elk.spacing.portPort` as the client sets it.
pub const PORT_SPACING: f64 = 16.0;
/// The client's `MIN_NODE_WIDTH`/`MIN_NODE_HEIGHT` (`elk.nodeSize.minimum`).
pub const MIN_NODE_WIDTH: f64 = 120.0;
pub const MIN_NODE_HEIGHT: f64 = 40.0;

/// The side length `n` ports need under ELK's port spacing.
pub fn ports_extent(n: usize) -> f64 {
    if n == 0 {
        0.0
    } else {
        n as f64 * PORT_SIZE + (n as f64 + 1.0) * PORT_SPACING
    }
}

/// Relative width margin on every label (8 %).
pub const WIDTH_MARGIN: f64 = 0.08;
/// Absolute width margin on every label, in px.
pub const WIDTH_PAD: f64 = 2.0;
/// Line box height as a multiple of the font size.
pub const LINE_FACTOR: f64 = 1.25;

/// The horizontal `elk.nodeLabels.padding` the client sets (`left=8,right=8`).
pub const NODE_LABELS_PADDING_H: f64 = 8.0;
/// The client's `NODE_VBOX`.
pub const NODE_PADDING_TOP: f64 = 4.0;
pub const NODE_PADDING_BOTTOM: f64 = 4.0;
pub const NODE_PADDING_LEFT: f64 = 0.0;
pub const NODE_PADDING_RIGHT: f64 = 0.0;
pub const NODE_VGAP: f64 = 1.0;
/// The client's `COMPARTMENT_VBOX`.
pub const COMPARTMENT_PADDING_TOP: f64 = 4.0;
pub const COMPARTMENT_PADDING_BOTTOM: f64 = 4.0;
pub const COMPARTMENT_PADDING_LEFT: f64 = 8.0;
pub const COMPARTMENT_PADDING_RIGHT: f64 = 8.0;
pub const COMPARTMENT_VGAP: f64 = 2.0;

/// The role of a label, which fixes its font (`views.tsx`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LabelRole {
    /// A node's name (12px bold; italic when the node is abstract).
    Name,
    /// `«stereotype»` (9px italic).
    Stereotype,
    /// `«banner»` for an applied stereotype (9px italic).
    Banner,
    /// One compartment line (10px).
    Line,
    /// A port's name (9px).
    PortName,
    /// An edge's `«keyword»` (10px italic).
    EdgeKeyword,
    /// An edge's label text (10px).
    EdgeLabel,
    /// A free IR label (12px).
    Free,
}

impl LabelRole {
    /// `(font size, bold, italic)`.
    pub fn font(&self) -> (f64, bool, bool) {
        match self {
            LabelRole::Name => (12.0, true, false),
            LabelRole::Stereotype | LabelRole::Banner => (9.0, false, true),
            LabelRole::Line => (10.0, false, false),
            LabelRole::PortName => (9.0, false, false),
            LabelRole::EdgeKeyword => (10.0, false, true),
            LabelRole::EdgeLabel => (10.0, false, false),
            LabelRole::Free => (12.0, false, false),
        }
    }

    /// The client's `LabelRole` spelling.
    pub fn as_str(&self) -> &'static str {
        match self {
            LabelRole::Name => "name",
            LabelRole::Stereotype => "stereotype",
            LabelRole::Banner => "banner",
            LabelRole::Line => "line",
            LabelRole::PortName => "name",
            LabelRole::EdgeKeyword => "keyword",
            LabelRole::EdgeLabel => "edge",
            LabelRole::Free => "free",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Size {
    pub w: f64,
    pub h: f64,
}

/// One label box: the client's synthetic label element, with the text it
/// shows, its font role, its box and its `vbox` position relative to its
/// owner (the node or compartment it stacks in; edge labels have none).
#[derive(Debug, Clone, PartialEq)]
pub struct LabelBox {
    /// The sprotty element id (`<node>-label`, `<node>-stereotype`,
    /// `<node>-banner-<i>`, `<compartment>-line-<i>`, `<edge>-keyword`,
    /// `<edge>-label`, or the IR id of a free label).
    pub id: String,
    pub text: String,
    pub role: LabelRole,
    /// Position inside the owner's box (`vbox`), when stacked.
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// A compartment's box inside its node (`vbox` position and size).
#[derive(Debug, Clone, PartialEq)]
pub struct CompartmentBox {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// The sizing of one IR node.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NodeSizing {
    /// The node's own box as the client would have measured it: the label
    /// stack for a node, 12×12 for a port, the line stack for a compartment,
    /// the text box for a free label. Pins are not applied here.
    pub size: Size,
    /// The ELK label children of a node or port, in the client's order
    /// (stereotype, banners, name, free labels); a compartment's lines.
    pub labels: Vec<LabelBox>,
    /// A node's compartment children at their `vbox` offsets.
    pub compartments: Vec<CompartmentBox>,
}

/// Sizes for a whole graph, keyed by IR node id and edge id.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Sizes {
    pub nodes: BTreeMap<String, NodeSizing>,
    /// Per edge: its `«keyword»` and label boxes (either may be absent).
    pub edges: BTreeMap<String, Vec<LabelBox>>,
}

impl Sizes {
    pub fn node(&self, id: &str) -> Option<&NodeSizing> {
        self.nodes.get(id)
    }

    /// The measured box of a node, `(0, 0)` for an unknown id.
    pub fn size_of(&self, id: &str) -> Size {
        self.nodes.get(id).map(|n| n.size).unwrap_or_default()
    }

    /// Every label box of the graph with its owner id, in sizing order.
    pub fn all_labels(&self) -> impl Iterator<Item = (&str, &LabelBox)> {
        let nodes = self.nodes.iter().flat_map(|(id, n)| n.labels.iter().map(move |l| (id.as_str(), l)));
        let edges = self.edges.iter().flat_map(|(id, ls)| ls.iter().map(move |l| (id.as_str(), l)));
        nodes.chain(edges)
    }
}

/// The box of one text run at `role`'s font, margin included.
pub fn label_size(metrics: &dyn TextMetrics, text: &str, role: LabelRole) -> Size {
    let (font, bold, _italic) = role.font();
    let advance = metrics.advance_width(text, font, bold);
    Size { w: (advance * (1.0 + WIDTH_MARGIN) + WIDTH_PAD).ceil(), h: (font * LINE_FACTOR).ceil() }
}

fn label(metrics: &dyn TextMetrics, id: String, text: String, role: LabelRole) -> LabelBox {
    let s = label_size(metrics, &text, role);
    LabelBox { id, text, role, x: 0.0, y: 0.0, w: s.w, h: s.h }
}

/// `(top, bottom, left, right)` paddings of a `vbox`.
type Padding = (f64, f64, f64, f64);

/// Stack boxes of the given `(w, h)` top-down as sprotty's `vbox` with
/// `hAlign: left` does; returns each box's position and the container size.
fn vbox(boxes: &[(f64, f64)], pad: Padding, vgap: f64) -> (Vec<(f64, f64)>, Size) {
    let (top, bottom, left, right) = pad;
    let mut y = top;
    let mut max_w: f64 = 0.0;
    let mut total_h: f64 = 0.0;
    let mut positions = Vec::with_capacity(boxes.len());
    for (i, (w, h)) in boxes.iter().enumerate() {
        if i > 0 {
            y += vgap;
            total_h += vgap;
        }
        positions.push((left, y));
        y += h;
        total_h += h;
        max_w = max_w.max(*w);
    }
    // sprotty leaves a childless container's bounds untouched; a node always
    // has its name label, so only a compartment with no lines gets here —
    // give it its paddings.
    (positions, Size { w: max_w + left + right, h: total_h + top + bottom })
}

fn compartment_sizing(metrics: &dyn TextMetrics, node: &Node) -> NodeSizing {
    let mut labels: Vec<LabelBox> = node
        .lines
        .iter()
        .enumerate()
        .map(|(i, line)| label(metrics, format!("{}-line-{i}", node.id), line.clone(), LabelRole::Line))
        .collect();
    let boxes: Vec<(f64, f64)> = labels.iter().map(|l| (l.w, l.h)).collect();
    let (positions, size) = vbox(
        &boxes,
        (COMPARTMENT_PADDING_TOP, COMPARTMENT_PADDING_BOTTOM, COMPARTMENT_PADDING_LEFT, COMPARTMENT_PADDING_RIGHT),
        COMPARTMENT_VGAP,
    );
    for (l, (x, y)) in labels.iter_mut().zip(positions) {
        l.x = x;
        l.y = y;
    }
    NodeSizing { size, labels, compartments: Vec::new() }
}

fn port_sizing(metrics: &dyn TextMetrics, node: &Node) -> NodeSizing {
    let name = label(metrics, format!("{}-label", node.id), node.label.clone(), LabelRole::PortName);
    NodeSizing { size: Size { w: PORT_SIZE, h: PORT_SIZE }, labels: vec![name], compartments: Vec::new() }
}

fn free_label_sizing(metrics: &dyn TextMetrics, node: &Node) -> NodeSizing {
    let s = label_size(metrics, &node.label, LabelRole::Free);
    NodeSizing { size: s, labels: Vec::new(), compartments: Vec::new() }
}

/// Pseudostate and control-node glyphs (`REQ-TRS-VIS-018`/`-019`) have a
/// fixed box: initial/final 20×20, fork/join bars 60×6, decision/merge
/// diamonds 28×28. The initial and final circles carry no text; a fork, join,
/// decision or merge shows its label (a control node's name, a decision's
/// condition) beside the glyph, to the right and vertically centred —
/// the layout engines place it `OUTSIDE` the node.
pub fn glyph_size(kind: NodeKind) -> Option<Size> {
    Some(match kind {
        NodeKind::Initial | NodeKind::Final => Size { w: 20.0, h: 20.0 },
        NodeKind::Fork | NodeKind::Join => Size { w: 60.0, h: 6.0 },
        NodeKind::Decision | NodeKind::Merge => Size { w: 28.0, h: 28.0 },
        _ => return None,
    })
}

/// Horizontal gap between a glyph and its outside label.
pub const GLYPH_LABEL_GAP: f64 = 6.0;

fn glyph_sizing(metrics: &dyn TextMetrics, node: &Node, size: Size) -> NodeSizing {
    let mut labels = Vec::new();
    if !matches!(node.kind, NodeKind::Initial | NodeKind::Final) && !node.label.trim().is_empty() {
        let mut l = label(metrics, format!("{}-label", node.id), node.label.clone(), LabelRole::Line);
        l.x = size.w + GLYPH_LABEL_GAP;
        l.y = ((size.h - l.h) / 2.0).round();
        labels.push(l);
    }
    NodeSizing { size, labels, compartments: Vec::new() }
}

/// Size one node from its already-sized children (compartments, free labels).
fn node_sizing(metrics: &dyn TextMetrics, graph: &DiagramGraph, node: &Node, sized: &BTreeMap<String, NodeSizing>) -> NodeSizing {
    let mut labels: Vec<LabelBox> = Vec::new();
    if let Some(st) = &node.stereotype {
        labels.push(label(metrics, format!("{}-stereotype", node.id), format!("«{st}»"), LabelRole::Stereotype));
    }
    for (i, b) in node.banners.iter().enumerate() {
        labels.push(label(metrics, format!("{}-banner-{i}", node.id), format!("«{b}»"), LabelRole::Banner));
    }
    labels.push(label(metrics, format!("{}-label", node.id), node.label.clone(), LabelRole::Name));
    // The children in declaration order: compartments and free labels stack;
    // ports and nested nodes do not.
    enum Child {
        Label(usize),
        Compartment(usize),
    }
    let mut compartments: Vec<CompartmentBox> = Vec::new();
    let mut order: Vec<Child> = (0..labels.len()).map(Child::Label).collect();
    for child in graph.children_of(&node.id) {
        match child.kind {
            NodeKind::Compartment => {
                let s = sized.get(&child.id).map(|n| n.size).unwrap_or_default();
                compartments.push(CompartmentBox { id: child.id.clone(), x: 0.0, y: 0.0, w: s.w, h: s.h });
                order.push(Child::Compartment(compartments.len() - 1));
            }
            NodeKind::Label => {
                let s = sized.get(&child.id).map(|n| n.size).unwrap_or_default();
                labels.push(LabelBox { id: child.id.clone(), text: child.label.clone(), role: LabelRole::Free, x: 0.0, y: 0.0, w: s.w, h: s.h });
                order.push(Child::Label(labels.len() - 1));
            }
            _ => {}
        }
    }
    let boxes: Vec<(f64, f64)> = order
        .iter()
        .map(|c| match c {
            Child::Label(i) => (labels[*i].w, labels[*i].h),
            Child::Compartment(i) => (compartments[*i].w, compartments[*i].h),
        })
        .collect();
    let (positions, mut size) = vbox(&boxes, (NODE_PADDING_TOP, NODE_PADDING_BOTTOM, NODE_PADDING_LEFT, NODE_PADDING_RIGHT), NODE_VGAP);
    let compound = graph.children_of(&node.id).any(|c| sprotty_type(c.kind) == TYPE_NODE);
    if !compound {
        // A leaf is exactly its carried size (module doc): the client's
        // minimum, and room for its ports on the sides they use.
        let (mut ew, mut ns) = (0usize, 0usize);
        for p in graph.children_of(&node.id).filter(|c| c.kind == NodeKind::Port) {
            match port_side(p) {
                Some("east") | Some("west") => ew += 1,
                Some("north") | Some("south") => ns += 1,
                _ => {}
            }
        }
        // ELK centres the labels inside `elk.nodeLabels.padding` (8 a side):
        // give the stack that room so a label never overhangs its box.
        size.w = (size.w + 2.0 * NODE_LABELS_PADDING_H).max(MIN_NODE_WIDTH).max(ports_extent(ns));
        size.h = size.h.max(MIN_NODE_HEIGHT).max(ports_extent(ew));
    }
    for (c, (x, y)) in order.iter().zip(positions) {
        match c {
            Child::Label(i) => {
                labels[*i].x = x;
                labels[*i].y = y;
            }
            Child::Compartment(i) => {
                compartments[*i].x = x;
                compartments[*i].y = y;
            }
        }
    }
    NodeSizing { size, labels, compartments }
}

fn edge_labels(metrics: &dyn TextMetrics, edge: &Edge) -> Vec<LabelBox> {
    let style = edge_style(edge.kind);
    let mut out = Vec::new();
    if let Some(k) = &style.keyword {
        // The client shows `=` bare and wraps every other keyword in guillemets
        // (the style already carries them for the requirement relationships).
        let text = if k == "=" { "=".to_string() } else if k.starts_with('«') { k.clone() } else { format!("«{k}»") };
        out.push(label(metrics, format!("{}-keyword", edge.id), text, LabelRole::EdgeKeyword));
    }
    if let Some(l) = &edge.label {
        out.push(label(metrics, format!("{}-label", edge.id), l.clone(), LabelRole::EdgeLabel));
    }
    out
}

/// Size every node, port, compartment and label of `graph` (see the module
/// doc). Pure: the same graph and metrics always give the same sizes.
pub fn size_graph(graph: &DiagramGraph, metrics: &dyn TextMetrics) -> Sizes {
    let mut nodes: BTreeMap<String, NodeSizing> = BTreeMap::new();
    // Leaves first (compartments, ports, free labels), then the nodes that
    // stack them; a node's own size never depends on nested nodes, so one
    // pass over the kinds suffices.
    for n in &graph.nodes {
        match n.kind {
            NodeKind::Compartment => {
                nodes.insert(n.id.clone(), compartment_sizing(metrics, n));
            }
            NodeKind::Port => {
                nodes.insert(n.id.clone(), port_sizing(metrics, n));
            }
            NodeKind::Label => {
                nodes.insert(n.id.clone(), free_label_sizing(metrics, n));
            }
            _ => {}
        }
    }
    for n in &graph.nodes {
        if let Some(s) = glyph_size(n.kind) {
            nodes.insert(n.id.clone(), glyph_sizing(metrics, n, s));
        } else if !matches!(n.kind, NodeKind::Compartment | NodeKind::Port | NodeKind::Label) {
            let s = node_sizing(metrics, graph, n, &nodes);
            nodes.insert(n.id.clone(), s);
        }
    }
    let edges = graph.edges.iter().map(|e| (e.id.clone(), edge_labels(metrics, e))).collect();
    Sizes { nodes, edges }
}

/// [`size_graph`] with the process-wide diagram metrics.
pub fn size_graph_default(graph: &DiagramGraph) -> Sizes {
    size_graph(graph, super::metrics::diagram_metrics())
}

/// The size a node is carried with: a pin's `w`/`h` when it records both
/// (the browser's *Pin all* wrote ELK's final box, which wins over a fresh
/// measurement), else the measured box.
pub fn carried_size(node: &Node, sizes: &Sizes) -> Size {
    match node.pin {
        Some(p) if p.w.is_some() && p.h.is_some() => Size { w: p.w.unwrap_or(0.0), h: p.h.unwrap_or(0.0) },
        _ => sizes.size_of(&node.id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vis::ir::{DiagramKind, EdgeKind, PortDirection};
    use crate::vis::metrics::ApproxMetrics;

    fn node(id: &str, kind: NodeKind, parent: Option<&str>, label: &str) -> Node {
        Node {
            id: id.into(),
            element_ref: format!("Sys::{id}"),
            resolved: true,
            element_type: None,
            kind,
            label: label.into(),
            stereotype: None,
            parent: parent.map(str::to_string),
            direction: None,
            side: None,
            lines: vec![],
            is_abstract: false,
            pin: None,
            banners: vec![],
        }
    }

    /// With `ApproxMetrics` a label is `ceil(n × font × 0.58 × 1.08 + 2)` wide.
    fn approx_w(text: &str, font: f64) -> f64 {
        (text.chars().count() as f64 * font * 0.58 * 1.08 + 2.0).ceil()
    }

    #[test]
    fn label_boxes_carry_the_margin_and_the_line_factor() {
        let m = ApproxMetrics;
        let s = label_size(&m, "Engine", LabelRole::Name);
        assert_eq!(s.w, approx_w("Engine", 12.0));
        assert_eq!(s.h, 15.0, "ceil(12 × 1.25)");
        assert_eq!(label_size(&m, "«part»", LabelRole::Stereotype).h, 12.0, "ceil(9 × 1.25)");
        assert_eq!(label_size(&m, "x", LabelRole::Line).h, 13.0);
        assert!(label_size(&m, "Engine", LabelRole::Name).w > label_size(&m, "Engine", LabelRole::PortName).w);
    }

    #[test]
    fn a_block_stacks_stereotype_banners_name_and_compartment_like_the_client_vbox() {
        let m = ApproxMetrics;
        let mut g = DiagramGraph::empty(DiagramKind::Bdd, "D", "D", None);
        let mut a = node("a", NodeKind::Block, None, "Engine");
        a.stereotype = Some("part def".into());
        a.banners = vec!["Safety".into()];
        g.nodes.push(a);
        let mut c = node("a-compartment", NodeKind::Compartment, Some("a"), "");
        c.lines = vec!["mass : Real [kg]".into(), "port powerOut : PowerPort (out)".into()];
        g.nodes.push(c);
        let s = size_graph(&g, &m);
        let a = s.node("a").unwrap();
        let ids: Vec<&str> = a.labels.iter().map(|l| l.id.as_str()).collect();
        assert_eq!(ids, vec!["a-stereotype", "a-banner-0", "a-label"]);
        assert_eq!(a.labels[0].text, "«part def»");
        assert_eq!(a.labels[1].text, "«Safety»");
        assert_eq!(a.labels[2].role, LabelRole::Name);
        // vbox: y = 4, then +h+1 each.
        assert_eq!((a.labels[0].x, a.labels[0].y), (0.0, 4.0));
        assert_eq!(a.labels[1].y, 4.0 + 12.0 + 1.0);
        assert_eq!(a.labels[2].y, 4.0 + 12.0 + 1.0 + 12.0 + 1.0);
        let comp = &a.compartments[0];
        assert_eq!(comp.id, "a-compartment");
        assert_eq!(comp.y, a.labels[2].y + 15.0 + 1.0);
        // The compartment: 8 + widest line + 8 wide; 4 + 13 + 2 + 13 + 4 tall.
        let cs = s.node("a-compartment").unwrap();
        assert_eq!(cs.size.w, approx_w("port powerOut : PowerPort (out)", 10.0) + 16.0);
        assert_eq!(cs.size.h, 4.0 + 13.0 + 2.0 + 13.0 + 4.0);
        assert_eq!(cs.labels.len(), 2);
        assert_eq!(cs.labels[1].id, "a-compartment-line-1");
        assert_eq!((cs.labels[0].x, cs.labels[0].y), (8.0, 4.0));
        assert_eq!(cs.labels[1].y, 4.0 + 13.0 + 2.0);
        // The node: widest child plus ELK's label padding wide, stack tall.
        assert_eq!(a.size.w, (cs.size.w + 16.0).max(MIN_NODE_WIDTH));
        assert_eq!(a.size.h, 4.0 + 12.0 + 1.0 + 12.0 + 1.0 + 15.0 + 1.0 + cs.size.h + 4.0);
    }

    #[test]
    fn a_leaf_is_raised_to_the_minimum_and_to_what_its_ports_need() {
        let m = ApproxMetrics;
        let mut g = DiagramGraph::empty(DiagramKind::Ibd, "D", "D", None);
        g.nodes.push(node("b", NodeKind::Boundary, None, "Sys"));
        g.nodes.push(node("a", NodeKind::Block, Some("b"), "a"));
        for (i, dir) in [PortDirection::Out, PortDirection::Out, PortDirection::In].iter().enumerate() {
            let mut p = node(&format!("p{i}"), NodeKind::Port, Some("a"), "p");
            p.direction = Some(*dir);
            g.nodes.push(p);
        }
        let s = size_graph(&g, &m);
        let a = s.size_of("a");
        assert_eq!(a.w, MIN_NODE_WIDTH, "a short label is raised to the client's minimum width");
        assert_eq!(a.h, ports_extent(3), "two east and one west port: three on the east/west axis, 28×3+16");
        assert_eq!(ports_extent(1), 44.0);
        assert_eq!(ports_extent(0), 0.0);
        // The boundary (compound: it nests `a`) is only its label stack: ELK sizes it.
        assert!(s.size_of("b").w < MIN_NODE_WIDTH);
        assert!(s.size_of("b").h < MIN_NODE_HEIGHT);
        // A pin with both w and h is carried; one without is not.
        let mut pinned = g.nodes[1].clone();
        pinned.pin = Some(crate::vis::ir::Rect { x: 1.0, y: 2.0, w: Some(300.0), h: Some(90.0) });
        assert_eq!(carried_size(&pinned, &s), Size { w: 300.0, h: 90.0 });
        pinned.pin = Some(crate::vis::ir::Rect { x: 1.0, y: 2.0, w: Some(300.0), h: None });
        assert_eq!(carried_size(&pinned, &s), a);
    }

    #[test]
    fn ports_are_twelve_square_with_a_name_label_and_free_labels_are_text_boxes() {
        let m = ApproxMetrics;
        let mut g = DiagramGraph::empty(DiagramKind::Ibd, "D", "D", None);
        g.nodes.push(node("b", NodeKind::Boundary, None, "Sys"));
        let mut p = node("p", NodeKind::Port, Some("b"), "powerOut");
        p.direction = Some(PortDirection::Out);
        g.nodes.push(p);
        g.nodes.push(node("free", NodeKind::Label, None, "a note"));
        let s = size_graph(&g, &m);
        let p = s.node("p").unwrap();
        assert_eq!(p.size, Size { w: 12.0, h: 12.0 });
        assert_eq!(p.labels[0].id, "p-label");
        assert_eq!(p.labels[0].role, LabelRole::PortName);
        assert_eq!(p.labels[0].w, approx_w("powerOut", 9.0));
        let f = s.node("free").unwrap();
        assert_eq!(f.size.w, approx_w("a note", 12.0));
        assert!(f.labels.is_empty());
        // The boundary nests no node, so it is a leaf like the client sees it:
        // its label stack is raised to the minimum and to its one east port.
        let b = s.node("b").unwrap();
        assert_eq!(b.labels.len(), 1, "no stereotype: only the name label");
        assert_eq!(b.labels[0].y, 4.0);
        assert_eq!(b.size.h, ports_extent(1));
        assert_eq!(b.size.w, MIN_NODE_WIDTH);
    }

    #[test]
    fn edge_labels_follow_the_style_keyword_and_the_label() {
        let m = ApproxMetrics;
        let mut g = DiagramGraph::empty(DiagramKind::Requirement, "D", "D", None);
        for (id, kind, label) in [("e1", EdgeKind::Verify, Some("tc")), ("e2", EdgeKind::Binding, None), ("e3", EdgeKind::Flow, None)] {
            g.edges.push(Edge {
                id: id.into(),
                element_ref: None,
                source: "a".into(),
                target: "b".into(),
                kind,
                label: label.map(str::to_string),
                waypoints: None,
            });
        }
        let s = size_graph(&g, &m);
        let e1 = &s.edges["e1"];
        assert_eq!(e1.iter().map(|l| (l.id.as_str(), l.text.as_str())).collect::<Vec<_>>(), vec![("e1-keyword", "«verify»"), ("e1-label", "tc")]);
        assert_eq!(e1[0].role, LabelRole::EdgeKeyword);
        assert_eq!(s.edges["e2"].iter().map(|l| l.text.as_str()).collect::<Vec<_>>(), vec!["="]);
        assert!(s.edges["e3"].is_empty());
    }

    #[test]
    fn sizing_is_deterministic() {
        let m = ApproxMetrics;
        let mut g = DiagramGraph::empty(DiagramKind::Bdd, "D", "D", None);
        g.nodes.push(node("a", NodeKind::Block, None, "A"));
        assert_eq!(size_graph(&g, &m), size_graph(&g, &m));
    }
}
