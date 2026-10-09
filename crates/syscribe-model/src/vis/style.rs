//! The one visual language (`REQ-TRS-VIS-012`): fill, stroke, header, dash,
//! arrowhead and port glyph per node type / node kind / edge kind / port
//! direction, owned here so the sprotty client's views and the SVG writer
//! read the same values instead of each keeping its own table.
//!
//! The sprotty writer ([`super::sprotty`]) attaches a resolved [`NodeStyle`],
//! [`PortStyle`] or [`EdgeStyle`] to every element it emits; the client draws
//! what it is handed. Edge notation follows spec §8.16.8's per-kind tables
//! (hollow triangle for inheritance, filled diamond at the whole end for
//! composition, filled arrow for flow, dashed `=` for binding, nothing for a
//! connection, dashed open arrow with a `«keyword»` for the requirement
//! relationships).

use serde::{Deserialize, Serialize};

use super::ir::{EdgeKind, Node, NodeKind, PortDirection, Tone};

/// Text colour shared by every node.
pub const TEXT: &str = "#222";
/// Default edge stroke.
pub const EDGE_STROKE: &str = "#555";
/// Edge stroke for the IBD connector family (connection, flow, binding).
pub const EDGE_STROKE_CONNECTOR: &str = "#1f497d";
/// Edge stroke for `«verify»`.
pub const EDGE_STROKE_VERIFY: &str = "#3a6ea5";
/// Cross-tree constraints of a feature diagram (`REQ-TRS-FMED-001`).
pub const EDGE_STROKE_FEATURE_REQUIRES: &str = "#1d6fb8";
pub const EDGE_STROKE_FEATURE_EXCLUDES: &str = "#b3261e";
/// Edge stroke for `«allocate»`.
pub const EDGE_STROKE_ALLOCATION: &str = "#7a3ea5";
/// Edge stroke of the easiest attack path.
pub const EDGE_STROKE_CRITICAL: &str = "#b3261e";
/// Stroke width of a critical-path edge and of an emphasised node outline.
pub const EMPHASIS_WIDTH: f64 = 3.0;
/// Edge stroke width shared by every kind.
pub const EDGE_WIDTH: f64 = 1.4;

/// Resolved colours of one node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeStyle {
    pub fill: String,
    pub stroke: String,
    /// A tinted header band behind the stereotype/name, in this colour (the
    /// requirement and test-case boxes); `None` for a plain box.
    pub header_fill: Option<String>,
    pub text: String,
    /// Dashed outline: the node's reference did not resolve.
    pub dashed: bool,
    /// Outline width when it is not the writer's default (an emphasised
    /// [`NodeMark`](super::ir::NodeMark)).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<f64>,
}

/// Arrowhead vocabulary, serialised camelCase (`hollowTriangle`, …).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ArrowHead {
    None,
    Filled,
    Open,
    HollowTriangle,
    FilledDiamond,
    HollowDiamond,
    FilledCircle,
}

/// Resolved stroke, dash, arrowheads and keyword of one edge kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EdgeStyle {
    pub stroke: String,
    /// SVG `stroke-dasharray`, or `None` for a solid line.
    pub dash: Option<String>,
    pub width: f64,
    pub arrow_target: ArrowHead,
    /// The decoration at the source end — the diamond of a composition or
    /// aggregation sits here, at the whole/aggregate end (the generators make
    /// the owner the edge's source).
    pub arrow_source: ArrowHead,
    /// A `«keyword»` (or `=` for a binding) drawn at the edge's middle.
    pub keyword: Option<String>,
}

/// Resolved look of one port square.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortStyle {
    pub fill: String,
    pub stroke: String,
    /// `"in"`, `"out"`, `"inout"` or `"none"`: which direction glyph the
    /// client draws (`inout` is the half-filled square).
    pub glyph: &'static str,
}

fn by_element_type(t: &str) -> Option<(&'static str, &'static str, bool)> {
    // (fill, stroke, header)
    Some(match t {
        "PartDef" | "Part" | "ItemDef" | "Item" => ("#f5f5fa", "#3a3a4a", false),
        "PortDef" | "Port" => ("#eef3f8", "#1f497d", false),
        "InterfaceDef" | "Interface" => ("#e8f8f5", "#0e6655", false),
        "ConnectionDef" | "Connection" => ("#f4f4f4", "#555", false),
        "RequirementDef" | "Requirement" => ("#f9f7ff", "#4a0a6e", true),
        "TestCase" => ("#f0fff4", "#1e6b2e", true),
        "ActionDef" | "Action" => ("#fef9e7", "#9a7d0a", false),
        "StateDef" | "State" => ("#f3eefa", "#5b2c8e", false),
        "UseCaseDef" | "UseCase" => ("#fff8e1", "#8d6e00", false),
        "Allocation" => ("#f4ecf7", "#7d3c98", false),
        _ => return None,
    })
}

fn by_node_kind(kind: NodeKind) -> (&'static str, &'static str, bool) {
    match kind {
        NodeKind::Boundary | NodeKind::SystemBoundary | NodeKind::Swimlane => ("#f8f9fb", "#3a3a4a", false),
        NodeKind::Note => ("#fffde7", "#f9a825", false),
        NodeKind::Actor => ("#fff", "#333", false),
        NodeKind::Lifeline => ("#eef3f8", "#1f497d", false),
        NodeKind::Port => ("#eef3f8", "#1f497d", false),
        NodeKind::Requirement => ("#f9f7ff", "#4a0a6e", true),
        NodeKind::TestCase => ("#f0fff4", "#1e6b2e", true),
        NodeKind::State => ("#f3eefa", "#5b2c8e", false),
        NodeKind::Action => ("#fef9e7", "#9a7d0a", false),
        NodeKind::Fork | NodeKind::Join => ("#333", "#333", false),
        NodeKind::Decision | NodeKind::Merge => ("#fff", "#333", false),
        NodeKind::UseCase => ("#fff8e1", "#8d6e00", false),
        // FaultTree / AttackTree / SafetyCase (GH #223).
        NodeKind::GateAnd | NodeKind::GateOr | NodeKind::GateXor | NodeKind::GateNot | NodeKind::GateInhibit => ("#eef3f8", "#1f497d", false),
        NodeKind::EventBasic => ("#ffffff", "#44546a", false),
        NodeKind::EventUndeveloped => ("#fffdf5", "#8d6e00", false),
        NodeKind::EventHouse => ("#f4f4f4", "#555", false),
        NodeKind::Step => ("#fdf3f0", "#9c3b22", false),
        NodeKind::Goal | NodeKind::UndevelopedGoal => ("#eef6ff", "#1d4e89", false),
        NodeKind::Strategy => ("#f3eefa", "#5b2c8e", false),
        NodeKind::Solution => ("#f0fff4", "#1e6b2e", false),
        NodeKind::Context => ("#f5f5f5", "#666", false),
        NodeKind::Justification | NodeKind::Assumption => ("#fffde7", "#8d6e00", false),
        _ => ("#f5f5fa", "#666", false),
    }
}

/// Whether the node kind belongs to the safety diagrams, whose look is fixed
/// by its role: the kind wins over the element type there.
fn is_safety_kind(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::GateAnd
            | NodeKind::GateOr
            | NodeKind::GateXor
            | NodeKind::GateNot
            | NodeKind::GateInhibit
            | NodeKind::EventBasic
            | NodeKind::EventUndeveloped
            | NodeKind::EventHouse
            | NodeKind::Step
            | NodeKind::Goal
            | NodeKind::UndevelopedGoal
            | NodeKind::Strategy
            | NodeKind::Solution
            | NodeKind::Context
            | NodeKind::Justification
            | NodeKind::Assumption
    )
}

/// `(fill, stroke)` a [`Tone`] paints over a node's base colours.
pub fn tone_colors(tone: Tone) -> (&'static str, &'static str) {
    match tone {
        Tone::Ok => ("#e3f5e8", "#1e8a3c"),
        Tone::Warn => ("#fff4d6", "#b7791f"),
        Tone::Bad => ("#fdecea", "#b3261e"),
        Tone::Neutral => ("#f1f2f4", "#8a8f98"),
    }
}

/// The style of a node: by its resolved element type first, then by its
/// [`NodeKind`] for an untyped node. Dashed when the reference is unresolved.
pub fn node_style(node: &Node) -> NodeStyle {
    let (mut fill, mut stroke, header) = if is_safety_kind(node.kind) {
        by_node_kind(node.kind)
    } else {
        node.element_type.as_deref().and_then(by_element_type).unwrap_or_else(|| by_node_kind(node.kind))
    };
    // A mark's tone repaints the node (GH #223); an emphasised mark draws it heavier.
    let mut stroke_width = None;
    if let Some(m) = node.mark.as_ref().filter(|m| !m.is_detail_only()) {
        let (f, s) = tone_colors(m.tone);
        fill = f;
        stroke = s;
        if m.emphasis {
            stroke_width = Some(EMPHASIS_WIDTH);
        }
    }
    NodeStyle {
        fill: fill.to_string(),
        stroke: stroke.to_string(),
        header_fill: header.then(|| stroke.to_string()),
        text: TEXT.to_string(),
        dashed: !node.resolved,
        stroke_width,
    }
}

/// The style of an edge kind, per spec §8.16.8's notation tables.
pub fn edge_style(kind: EdgeKind) -> EdgeStyle {
    use ArrowHead as A;
    use EdgeKind as K;
    // (stroke, dash, arrow_target, arrow_source, keyword)
    let (stroke, dash, target, source, keyword): (&str, Option<&str>, ArrowHead, ArrowHead, Option<&str>) = match kind {
        K::Connection => (EDGE_STROKE_CONNECTOR, None, A::None, A::None, None),
        K::Flow => (EDGE_STROKE_CONNECTOR, None, A::Filled, A::None, None),
        K::Binding => (EDGE_STROKE_CONNECTOR, Some("4,4"), A::None, A::None, Some("=")),
        K::Succession => (EDGE_STROKE, Some("6,3"), A::Filled, A::None, None),
        K::Inheritance => (EDGE_STROKE, None, A::HollowTriangle, A::None, None),
        K::Association => (EDGE_STROKE, None, A::None, A::None, None),
        K::Composition => (EDGE_STROKE, None, A::None, A::FilledDiamond, None),
        K::Aggregation => (EDGE_STROKE, None, A::None, A::HollowDiamond, None),
        K::Dependency => (EDGE_STROKE, Some("6,3"), A::Open, A::None, None),
        K::Message => (EDGE_STROKE, None, A::Filled, A::None, None),
        K::Return => (EDGE_STROKE, Some("6,3"), A::Open, A::None, None),
        K::Create => (EDGE_STROKE, Some("4,4"), A::Open, A::None, None),
        K::Destroy => (EDGE_STROKE, None, A::None, A::None, None),
        K::Transition => (EDGE_STROKE, None, A::Filled, A::None, None),
        K::Containment => (EDGE_STROKE, None, A::None, A::None, None),
        K::Derive => (EDGE_STROKE, Some("6,3"), A::Open, A::None, Some("«deriveReqt»")),
        K::Satisfy => (EDGE_STROKE, Some("6,3"), A::Open, A::None, Some("«satisfy»")),
        K::Verify => (EDGE_STROKE_VERIFY, Some("6,3"), A::Open, A::None, Some("«verify»")),
        K::Refine => (EDGE_STROKE, Some("6,3"), A::Open, A::None, Some("«refine»")),
        K::Trace => (EDGE_STROKE, Some("6,3"), A::Open, A::None, Some("«trace»")),
        K::Copy => (EDGE_STROKE, Some("6,3"), A::Open, A::None, Some("«copy»")),
        K::Allocation => (EDGE_STROKE_ALLOCATION, Some("8,4"), A::Open, A::None, Some("«allocate»")),
        K::Include => (EDGE_STROKE, Some("6,3"), A::Open, A::None, Some("«include»")),
        K::Extend => (EDGE_STROKE, Some("6,3"), A::Open, A::None, Some("«extend»")),
        K::FeatureChild => (EDGE_STROKE, None, A::None, A::None, None),
        K::Requires => (EDGE_STROKE_FEATURE_REQUIRES, Some("6,3"), A::Filled, A::None, Some("requires")),
        K::Excludes => (EDGE_STROKE_FEATURE_EXCLUDES, Some("6,3"), A::Open, A::Open, Some("excludes")),
        // The arrow sits on the gate end and points into the gate: a fault propagates up.
        K::GateInput => (EDGE_STROKE, None, A::None, A::Filled, None),
        K::CriticalPath => (EDGE_STROKE_CRITICAL, None, A::None, A::Filled, None),
        K::SupportedBy => (EDGE_STROKE, None, A::Filled, A::None, None),
        K::InContextOf => (EDGE_STROKE, None, A::HollowTriangle, A::None, None),
    };
    EdgeStyle {
        stroke: stroke.to_string(),
        dash: dash.map(str::to_string),
        width: if kind == K::CriticalPath { EMPHASIS_WIDTH } else { EDGE_WIDTH },
        arrow_target: target,
        arrow_source: source,
        keyword: keyword.map(str::to_string),
    }
}

/// The style of a port square by direction: white for `in`, dark for `out`,
/// half-filled (`inout` glyph, drawn by the client) for `inout`, grey for an
/// undirected port.
pub fn port_style(direction: Option<PortDirection>) -> PortStyle {
    let stroke = "#1f497d";
    let (fill, glyph) = match direction {
        Some(PortDirection::In) => ("#fff", "in"),
        Some(PortDirection::Out) => ("#333", "out"),
        Some(PortDirection::Inout) => ("#fff", "inout"),
        None => ("#ddd", "none"),
    };
    PortStyle { fill: fill.to_string(), stroke: stroke.to_string(), glyph }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(element_type: Option<&str>, kind: NodeKind, resolved: bool) -> Node {
        Node {
            id: "n".into(),
            element_ref: "X::n".into(),
            resolved,
            element_type: element_type.map(str::to_string),
            kind,
            label: "n".into(),
            stereotype: None,
            parent: None,
            direction: None,
            side: None,
            lines: vec![],
            is_abstract: false,
            pin: None,
            banners: vec![],
            feature: None,
            mark: None,
        }
    }

    #[test]
    fn node_style_prefers_element_type_then_kind_and_dashes_unresolved() {
        let s = node_style(&node(Some("PartDef"), NodeKind::Block, true));
        assert_eq!((s.fill.as_str(), s.stroke.as_str(), s.header_fill.as_deref(), s.text.as_str(), s.dashed), ("#f5f5fa", "#3a3a4a", None, "#222", false));
        assert_eq!(node_style(&node(Some("Part"), NodeKind::Block, true)), s);
        let r = node_style(&node(Some("Requirement"), NodeKind::Requirement, true));
        assert_eq!((r.fill.as_str(), r.stroke.as_str(), r.header_fill.as_deref()), ("#f9f7ff", "#4a0a6e", Some("#4a0a6e")));
        let t = node_style(&node(Some("TestCase"), NodeKind::TestCase, true));
        assert_eq!((t.fill.as_str(), t.stroke.as_str(), t.header_fill.as_deref()), ("#f0fff4", "#1e6b2e", Some("#1e6b2e")));
        for (ty, fill, stroke) in [
            ("PortDef", "#eef3f8", "#1f497d"),
            ("Interface", "#e8f8f5", "#0e6655"),
            ("ConnectionDef", "#f4f4f4", "#555"),
            ("ActionDef", "#fef9e7", "#9a7d0a"),
            ("State", "#f3eefa", "#5b2c8e"),
            ("UseCaseDef", "#fff8e1", "#8d6e00"),
            ("Allocation", "#f4ecf7", "#7d3c98"),
        ] {
            let s = node_style(&node(Some(ty), NodeKind::Block, true));
            assert_eq!((s.fill.as_str(), s.stroke.as_str(), s.header_fill.is_none()), (fill, stroke, true), "{ty}");
        }
        // The element type wins over the role: a Package drawn as a boundary is
        // untyped for styling purposes and falls back to the kind.
        let b = node_style(&node(Some("Package"), NodeKind::Boundary, true));
        assert_eq!((b.fill.as_str(), b.stroke.as_str()), ("#f8f9fb", "#3a3a4a"));
        for (kind, fill, stroke) in [
            (NodeKind::SystemBoundary, "#f8f9fb", "#3a3a4a"),
            (NodeKind::Swimlane, "#f8f9fb", "#3a3a4a"),
            (NodeKind::Note, "#fffde7", "#f9a825"),
            (NodeKind::Actor, "#fff", "#333"),
            (NodeKind::Lifeline, "#eef3f8", "#1f497d"),
            (NodeKind::Block, "#f5f5fa", "#666"),
            (NodeKind::Fragment, "#f5f5fa", "#666"),
        ] {
            let s = node_style(&node(None, kind, true));
            assert_eq!((s.fill.as_str(), s.stroke.as_str()), (fill, stroke), "{kind:?}");
        }
        assert!(node_style(&node(Some("PartDef"), NodeKind::Block, false)).dashed);
    }

    #[test]
    fn edge_style_follows_the_spec_tables_row_by_row() {
        use ArrowHead as A;
        let rows: [(EdgeKind, &str, Option<&str>, ArrowHead, ArrowHead, Option<&str>); 24] = [
            (EdgeKind::Connection, "#1f497d", None, A::None, A::None, None),
            (EdgeKind::Flow, "#1f497d", None, A::Filled, A::None, None),
            (EdgeKind::Binding, "#1f497d", Some("4,4"), A::None, A::None, Some("=")),
            (EdgeKind::Succession, "#555", Some("6,3"), A::Filled, A::None, None),
            (EdgeKind::Inheritance, "#555", None, A::HollowTriangle, A::None, None),
            (EdgeKind::Association, "#555", None, A::None, A::None, None),
            (EdgeKind::Composition, "#555", None, A::None, A::FilledDiamond, None),
            (EdgeKind::Aggregation, "#555", None, A::None, A::HollowDiamond, None),
            (EdgeKind::Dependency, "#555", Some("6,3"), A::Open, A::None, None),
            (EdgeKind::Message, "#555", None, A::Filled, A::None, None),
            (EdgeKind::Return, "#555", Some("6,3"), A::Open, A::None, None),
            (EdgeKind::Create, "#555", Some("4,4"), A::Open, A::None, None),
            (EdgeKind::Destroy, "#555", None, A::None, A::None, None),
            (EdgeKind::Transition, "#555", None, A::Filled, A::None, None),
            (EdgeKind::Containment, "#555", None, A::None, A::None, None),
            (EdgeKind::Derive, "#555", Some("6,3"), A::Open, A::None, Some("«deriveReqt»")),
            (EdgeKind::Satisfy, "#555", Some("6,3"), A::Open, A::None, Some("«satisfy»")),
            (EdgeKind::Verify, "#3a6ea5", Some("6,3"), A::Open, A::None, Some("«verify»")),
            (EdgeKind::Refine, "#555", Some("6,3"), A::Open, A::None, Some("«refine»")),
            (EdgeKind::Trace, "#555", Some("6,3"), A::Open, A::None, Some("«trace»")),
            (EdgeKind::Copy, "#555", Some("6,3"), A::Open, A::None, Some("«copy»")),
            (EdgeKind::Allocation, "#7a3ea5", Some("8,4"), A::Open, A::None, Some("«allocate»")),
            (EdgeKind::Include, "#555", Some("6,3"), A::Open, A::None, Some("«include»")),
            (EdgeKind::Extend, "#555", Some("6,3"), A::Open, A::None, Some("«extend»")),
        ];
        for (kind, stroke, dash, target, source, keyword) in rows {
            let s = edge_style(kind);
            assert_eq!(s.stroke, stroke, "{kind:?} stroke");
            assert_eq!(s.dash.as_deref(), dash, "{kind:?} dash");
            assert_eq!(s.arrow_target, target, "{kind:?} target");
            assert_eq!(s.arrow_source, source, "{kind:?} source");
            assert_eq!(s.keyword.as_deref(), keyword, "{kind:?} keyword");
            assert_eq!(s.width, EDGE_WIDTH, "{kind:?} width");
        }
    }

    #[test]
    fn port_style_by_direction() {
        let i = port_style(Some(PortDirection::In));
        assert_eq!((i.fill.as_str(), i.stroke.as_str(), i.glyph), ("#fff", "#1f497d", "in"));
        let o = port_style(Some(PortDirection::Out));
        assert_eq!((o.fill.as_str(), o.glyph), ("#333", "out"));
        let io = port_style(Some(PortDirection::Inout));
        assert_eq!((io.fill.as_str(), io.glyph), ("#fff", "inout"));
        let n = port_style(None);
        assert_eq!((n.fill.as_str(), n.glyph), ("#ddd", "none"));
    }

    #[test]
    fn styles_serialise_camel_case() {
        let j = serde_json::to_value(edge_style(EdgeKind::Composition)).unwrap();
        assert_eq!(j["arrowSource"], "filledDiamond");
        assert_eq!(j["arrowTarget"], "none");
        assert_eq!(j["dash"], serde_json::Value::Null);
        assert_eq!(j["keyword"], serde_json::Value::Null);
        let j = serde_json::to_value(edge_style(EdgeKind::Inheritance)).unwrap();
        assert_eq!(j["arrowTarget"], "hollowTriangle");
        let j = serde_json::to_value(node_style(&node(None, NodeKind::Block, true))).unwrap();
        assert_eq!(j["headerFill"], serde_json::Value::Null);
        assert_eq!(j["dashed"], false);
    }
}
