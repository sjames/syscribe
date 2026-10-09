//! Helpers shared by the analysis-graph generators (`Traceability`,
//! `ZoneConduit`, `ThreatGraph`; GH #223): node and edge construction, the
//! display name of an element and the tone roll-up.

use crate::element::{ElementType, RawElement, RawFrontmatter};

use super::super::ir::{derived_shape_id, DiagramGraph, Edge, EdgeKind, Node, NodeKind, NodeMark, Tone};
use super::short_name;

/// A stand-in `Package` for the whole model (its root, qualified name empty), the
/// subject of a model-wide analysis graph (`zones --format`, `cyber-risk --format`,
/// `hara trace`): every element is under it.
pub fn model_root_subject() -> RawElement {
    RawElement {
        qualified_name: String::new(),
        file_path: String::new(),
        frontmatter: RawFrontmatter { element_type: Some(ElementType::Package), ..Default::default() },
        doc: String::new(),
        parse_issue: None,
        derived: Default::default(),
        derive_findings: Vec::new(),
        locale_docs: Default::default(),
        about_notes: Vec::new(),
    }
}

/// An element's label: its `name`, else its stable `id`, else its short name.
pub(crate) fn display_name(e: &RawElement) -> String {
    e.frontmatter
        .name
        .clone()
        .or_else(|| e.frontmatter.id.clone())
        .unwrap_or_else(|| short_name(&e.qualified_name).to_string())
}

/// The id used to refer to an element in prose: its stable `id`, else its short name.
pub(crate) fn display_id(e: &RawElement) -> String {
    e.frontmatter.id.clone().unwrap_or_else(|| short_name(&e.qualified_name).to_string())
}

/// The names an `include:`/`exclude:` entry may use for `e`.
pub(crate) fn keys_of(e: &RawElement) -> Vec<String> {
    super::requirement::element_keys(e)
}

/// A node standing for the resolved element `e`, labelled the way the polished
/// safety diagrams are: the concise stable `id` is the label and the element's
/// name hangs under it, wrapped (`NodeMark::detail`); an element with no `id`
/// is labelled by its name.
pub(crate) fn node_for(e: &RawElement, kind: NodeKind, mark: Option<NodeMark>) -> Node {
    let name = display_name(e);
    let (label, mut mark) = match e.frontmatter.id.as_deref() {
        Some(id) if id != name => {
            let mut m = mark;
            m.get_or_insert_with(NodeMark::default).detail = Some(name);
            (id.to_string(), m)
        }
        _ => (name, mark),
    };
    if let Some(m) = mark.as_mut() {
        if m.detail.is_none() && m.is_detail_only() {
            mark = None;
        }
    }
    Node {
        id: derived_shape_id(&e.qualified_name),
        element_ref: e.qualified_name.clone(),
        resolved: true,
        element_type: e.frontmatter.element_type.as_ref().map(|t| t.name().to_string()),
        kind,
        label,
        stereotype: None,
        parent: None,
        direction: None,
        side: None,
        lines: Vec::new(),
        is_abstract: false,
        pin: None,
        banners: Vec::new(),
        feature: None,
        mark,
    }
}

/// A node for a reference that did not resolve: drawn dashed, labelled as written.
pub(crate) fn unresolved_node(text: &str, kind: NodeKind) -> Node {
    Node {
        id: derived_shape_id(&format!("unresolved::{text}")),
        element_ref: text.to_string(),
        resolved: false,
        element_type: None,
        kind,
        label: text.to_string(),
        stereotype: None,
        parent: None,
        direction: None,
        side: None,
        lines: Vec::new(),
        is_abstract: false,
        pin: None,
        banners: Vec::new(),
        feature: None,
        mark: Some(NodeMark { detail: None, status: Some("unresolved".to_string()), value: None, tone: Tone::Bad, badges: Vec::new(), emphasis: false }),
    }
}

/// Add `node` unless a node with that id is already there; returns its id.
pub(crate) fn add_node(graph: &mut DiagramGraph, node: Node) -> String {
    let id = node.id.clone();
    if !graph.nodes.iter().any(|n| n.id == id) {
        graph.nodes.push(node);
    }
    id
}

/// Add an edge once (same kind, source and target).
pub(crate) fn add_edge(graph: &mut DiagramGraph, kind: EdgeKind, source: &str, target: &str, element_ref: Option<String>, label: Option<String>) {
    let id = format!("e-{}-{source}-{target}", kind.as_str());
    if !graph.edges.iter().any(|e| e.id == id) {
        graph.edges.push(Edge { id, element_ref, source: source.to_string(), target: target.to_string(), kind, label, waypoints: None });
    }
}

/// Combine tones: any `Bad` wins, then `Warn`, then `Ok`; `Neutral` only when
/// nothing else contributed.
pub(crate) fn roll(tones: impl IntoIterator<Item = Tone>) -> Tone {
    let mut best = Tone::Neutral;
    for t in tones {
        best = match (best, t) {
            (Tone::Bad, _) | (_, Tone::Bad) => Tone::Bad,
            (Tone::Warn, _) | (_, Tone::Warn) => Tone::Warn,
            (Tone::Ok, _) | (_, Tone::Ok) => Tone::Ok,
            _ => Tone::Neutral,
        };
    }
    best
}

/// A mark with the given status, value, tone and badges.
pub(crate) fn mark(status: impl Into<String>, value: Option<String>, tone: Tone, badges: Vec<String>) -> NodeMark {
    NodeMark { detail: None, status: Some(status.into()), value, tone, badges, emphasis: false }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roll_prefers_the_worst_tone() {
        assert_eq!(roll([Tone::Ok, Tone::Ok]), Tone::Ok);
        assert_eq!(roll([Tone::Ok, Tone::Warn]), Tone::Warn);
        assert_eq!(roll([Tone::Warn, Tone::Bad, Tone::Ok]), Tone::Bad);
        assert_eq!(roll([Tone::Neutral, Tone::Ok]), Tone::Ok);
        assert_eq!(roll([]), Tone::Neutral);
    }
}
