//! IR → Graphviz DOT (GH #223).
//!
//! A pure function of the [`DiagramGraph`], written for the safety diagrams
//! (fault tree, attack tree, GSN) but total over every kind: containment is
//! ignored (DOT clusters add little to a tree), pins are ignored, and Graphviz
//! lays the picture out itself. Each node's shape follows its role (a gate a
//! `trapezium`/`invtriangle`/`hexagon`, a basic event an `ellipse`, a GSN
//! strategy a `parallelogram`, …), its text is the name followed by the
//! mark's status, value and badges, and its fill and outline are the tone
//! colours of [`super::style`]. Output is deterministic: nodes then edges in
//! IR order.

use super::ir::{DiagramGraph, Edge, EdgeKind, Node, NodeKind};
use super::style::{edge_style, node_style, ArrowHead};

/// A DOT-safe identifier: every character outside `[A-Za-z0-9_]` becomes `_`.
fn dot_id(id: &str) -> String {
    id.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' }).collect()
}

/// Text inside a double-quoted DOT string (`\n` stays the DOT line break).
fn quote(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', " ")
}

/// Graphviz reads only `#rgb`-less colours (`#rrggbb`): widen a three-digit hex.
fn color(c: &str) -> String {
    match c.strip_prefix('#') {
        Some(h) if h.len() == 3 => format!("#{}", h.chars().flat_map(|c| [c, c]).collect::<String>()),
        _ => c.to_string(),
    }
}

/// `text` broken at spaces into lines of at most `max` characters.
fn wrap(text: &str, max: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(l) if l.chars().count() + 1 + word.chars().count() <= max => {
                l.push(' ');
                l.push_str(word);
            }
            _ => lines.push(word.to_string()),
        }
    }
    lines
}

fn shape(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::GateAnd => "house",
        NodeKind::GateOr => "invhouse",
        NodeKind::GateXor | NodeKind::GateInhibit => "hexagon",
        NodeKind::GateNot => "invtriangle",
        NodeKind::EventBasic => "circle",
        NodeKind::Solution | NodeKind::Justification | NodeKind::Assumption => "ellipse",
        NodeKind::EventUndeveloped => "diamond",
        NodeKind::EventHouse => "pentagon",
        NodeKind::Strategy => "parallelogram",
        NodeKind::Context => "box",
        _ => "box",
    }
}

fn gate_word(kind: NodeKind) -> Option<&'static str> {
    match kind {
        NodeKind::GateAnd => Some("AND"),
        NodeKind::GateOr => Some("OR"),
        NodeKind::GateXor => Some("XOR"),
        NodeKind::GateNot => Some("NOT"),
        NodeKind::GateInhibit => Some("INHIBIT"),
        _ => None,
    }
}

fn label(node: &Node) -> String {
    let mut lines: Vec<String> = Vec::new();
    if let Some(w) = gate_word(node.kind) {
        lines.push(w.to_string());
    }
    lines.push(node.label.clone());
    if let Some(m) = &node.mark {
        if let Some(d) = &m.detail {
            lines.extend(wrap(d, 30));
        }
        lines.extend(m.status.clone());
        lines.extend(m.value.clone());
        if !m.badges.is_empty() {
            lines.push(m.badges.iter().map(|b| format!("[{b}]")).collect::<Vec<_>>().join(" "));
        }
    }
    lines.iter().map(|l| quote(l)).collect::<Vec<_>>().join("\\n")
}

fn node_line(node: &Node) -> String {
    let style = node_style(node);
    let mut attrs = vec![
        format!("label=\"{}\"", label(node)),
        format!("shape={}", shape(node.kind)),
        "style=\"filled\"".to_string(),
        format!("fillcolor=\"{}\"", color(&style.fill)),
        format!("color=\"{}\"", color(&style.stroke)),
    ];
    if node.kind == NodeKind::Context {
        attrs[2] = "style=\"filled,rounded\"".to_string();
    }
    if !node.resolved {
        attrs[2] = "style=\"filled,dashed\"".to_string();
    }
    if let Some(w) = style.stroke_width {
        attrs.push(format!("penwidth={w}"));
    }
    if node.resolved {
        attrs.push(format!("tooltip=\"{}\"", quote(&node.element_ref)));
    }
    let mut line = format!("  {} [{}];\n", dot_id(&node.id), attrs.join(", "));
    // GSN's undeveloped diamond hangs under the goal.
    if node.kind == NodeKind::UndevelopedGoal {
        let d = format!("{}__undeveloped", dot_id(&node.id));
        line.push_str(&format!(
            "  {d} [label=\"\", shape=diamond, width=0.3, height=0.2, fixedsize=true, style=\"filled\", fillcolor=\"#ffffff\", color=\"{}\"];\n  {} -> {d} [arrowhead=none, color=\"{}\"];\n",
            color(&style.stroke),
            dot_id(&node.id),
            color(&style.stroke)
        ));
    }
    line
}

fn arrow(a: ArrowHead) -> &'static str {
    match a {
        ArrowHead::None => "none",
        ArrowHead::Filled => "normal",
        ArrowHead::Open => "vee",
        ArrowHead::HollowTriangle => "empty",
        ArrowHead::FilledDiamond => "diamond",
        ArrowHead::HollowDiamond => "odiamond",
        ArrowHead::FilledCircle => "dot",
    }
}

fn edge_line(edge: &Edge) -> String {
    let s = edge_style(edge.kind);
    let mut attrs = vec![format!("color=\"{}\"", color(&s.stroke)), format!("arrowhead={}", arrow(s.arrow_target))];
    if s.arrow_source != ArrowHead::None {
        attrs.push(format!("arrowtail={}", arrow(s.arrow_source)));
        attrs.push("dir=both".to_string());
    }
    if s.dash.is_some() {
        attrs.push("style=dashed".to_string());
    }
    if edge.kind == EdgeKind::CriticalPath {
        attrs.push(format!("penwidth={}", s.width));
    }
    if let Some(l) = edge.label.clone().or(s.keyword) {
        attrs.push(format!("label=\"{}\"", quote(&l)));
    }
    format!("  {} -> {} [{}];\n", dot_id(&edge.source), dot_id(&edge.target), attrs.join(", "))
}

/// Render the graph as Graphviz DOT.
pub fn render_dot(graph: &DiagramGraph) -> String {
    let mut out = format!("digraph \"{}\" {{\n", quote(&graph.name));
    out.push_str("  rankdir=TB;\n  nodesep=0.4;\n  ranksep=0.55;\n  node [fontname=\"Helvetica\", fontsize=11];\n  edge [fontname=\"Helvetica\", fontsize=10];\n");
    for n in graph.nodes.iter().filter(|n| !matches!(n.kind, NodeKind::Compartment | NodeKind::Label)) {
        out.push_str(&node_line(n));
    }
    for e in &graph.edges {
        out.push_str(&edge_line(e));
    }
    out.push_str("}\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vis::ir::{DiagramKind, NodeMark, Tone};

    fn node(id: &str, kind: NodeKind, mark: Option<NodeMark>) -> Node {
        Node {
            id: id.into(),
            element_ref: format!("M::{id}"),
            resolved: true,
            element_type: None,
            kind,
            label: id.to_uppercase(),
            stereotype: None,
            parent: None,
            direction: None,
            side: None,
            lines: vec![],
            is_abstract: false,
            pin: None,
            banners: vec![],
            feature: None,
            mark,
        }
    }

    #[test]
    fn nodes_edges_marks_and_tones_are_written() {
        let mut g = DiagramGraph::empty(DiagramKind::FaultTree, "M::FT", "FT", None);
        g.nodes.push(node("top", NodeKind::GateOr, Some(NodeMark { detail: None, status: Some("top event".into()), value: Some("P 1.0e-6".into()), tone: Tone::Warn, badges: vec!["2 MCS".into()], emphasis: true })));
        g.nodes.push(node("a-1", NodeKind::EventBasic, None));
        g.edges.push(Edge { id: "e".into(), element_ref: None, source: "top".into(), target: "a-1".into(), kind: EdgeKind::GateInput, label: None, waypoints: None });
        let dot = render_dot(&g);
        assert!(dot.starts_with("digraph \"FT\" {"), "{dot}");
        assert!(dot.contains("top [label=\"OR\\nTOP\\ntop event\\nP 1.0e-6\\n[2 MCS]\", shape=invhouse"), "{dot}");
        assert!(dot.contains("fillcolor=\"#fff4d6\""), "warn tone fill: {dot}");
        assert!(dot.contains("penwidth=3"), "emphasis: {dot}");
        assert!(dot.contains("a_1 [label=\"A-1\", shape=circle"), "{dot}");
        assert!(dot.contains("top -> a_1 [color=\"#555555\", arrowhead=none, arrowtail=normal, dir=both];"), "{dot}");
    }
}
