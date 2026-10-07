//! IR → nested sprotty `SGraph` JSON (`REQ-TRS-VIS-006`, design §6.1).
//!
//! The JSON shape produced here is the contract between
//! `GET /api/diagrams/model/{*qname}` and the browser client in
//! `crates/syscribe-server/frontend/`. It is a pure function of the
//! [`DiagramGraph`]; nothing here reads frontmatter.
//!
//! ## Shape
//!
//! ```text
//! { id: "sysml-diagram", type: "graph", qualifiedName, diagramKind, subject?,
//!   layoutOptions: { "elk.algorithm", "elk.direction", "elk.hierarchyHandling"?,
//!                    "elk.portConstraints", "syscribe.reversedEdgeKinds": [...] },
//!   pinned: [nodeId, ...],
//!   children: [ <node>..., <edge>... ] }
//! ```
//!
//! Every IR node becomes one element whose sprotty `type` follows its
//! [`NodeKind`]: `"port"` for [`NodeKind::Port`], `"label"` for
//! [`NodeKind::Label`], `"compartment"` for [`NodeKind::Compartment`] and
//! `"node"` for everything else (boundary, block, state, requirement, …). The
//! IR's `kind` is carried verbatim as `kind` so the client can style by role.
//! Children are **nested** inside their parent's `children` (ports and
//! compartments included); edges are always **top-level** children of the
//! root, because sprotty resolves `sourceId`/`targetId` through the root
//! index regardless of nesting.
//!
//! Each `node`/`port` element also carries its synthetic label children
//! (`REQ-TRS-VIS-017`): `{ id: "<id>-stereotype", type: "label", text:
//! "«…»", role: "stereotype", size }`, one `{ id: "<id>-banner-<i>", role:
//! "banner" }` per applied stereotype, then `{ id: "<id>-label", role:
//! "name" }` — in that order, the order the client stacks them. A `port`
//! carries only its name label. A [`NodeKind::Compartment`] node is a
//! `compartment` element carrying `lines` and one `{ id:
//! "<id>-line-<i>", role: "line", size }` label child per line. An IR
//! [`NodeKind::Label`] node is itself a `label` element (`role: "free"`)
//! and carries its `text` directly. An `edge` carries its `«keyword»`
//! (`{ id: "<id>-keyword", type: "label:edge", role: "keyword" }`) and
//! `label` (`role: "edge"`) as `children` when it has them.
//!
//! ## Geometry
//!
//! `position` is present **only** when the IR node is pinned
//! ([`Node::pin`]). A pinned position of a *nested* node is relative to its
//! parent's origin — sprotty's own local-coordinate convention, and exactly
//! what `PATCH /api/diagrams/layout` writes back from a drag of a nested
//! node, so `layout:` round-trips without conversion. A `position` in the
//! JSON always means "a human pinned this".
//!
//! `size` is present on **every** node, port, compartment and label
//! (`REQ-TRS-VIS-017`): the box [`super::size::size_graph`] computes from the
//! shared text metrics — or, for a node whose pin records both `w` and `h`,
//! that pin's size ([`super::size::carried_size`]). The client treats it as
//! authoritative instead of measuring in the DOM, and the embedded ELK
//! ([`super::layout`]) starts from the same numbers, so both renderers lay
//! the same diagram out identically.
//!
//! `pinned` lists the ids of pinned nodes in declaration order, so the client
//! can tell "fixed by a human" from "placed by the layout engine". `derived`
//! is `true` when the content was generated from a `subject:` rather than
//! listed in `shapes:`.
//!
//! ## Style (`REQ-TRS-VIS-012`)
//!
//! Every `node` element carries `style: { fill, stroke, headerFill|null, text,
//! dashed }` and, when non-empty, `banners: ["Name", …]` (applied-stereotype
//! banners); every `port` element carries `style: { fill, stroke, glyph }` and
//! `side` (`north|east|south|west|null` — the IR's side, else derived from the
//! direction: in → west, out → east, inout → south); every `edge` carries
//! `style: { stroke, dash|null, width, arrowTarget, arrowSource, keyword|null }`
//! with [`super::style::ArrowHead`] values spelled camelCase. The client's
//! views read these and hold no colour table of their own.
//!
//! ## Ordering
//!
//! Output order is the IR's declaration order throughout (nodes, their
//! children, then edges) — never hash order — so the JSON is stable across
//! runs and diffable in tests.

use serde::Serialize;

use super::ir::{
    DiagramGraph, Edge, EdgeKind, LayoutAlgorithm, LayoutDirection, LayoutHints, Node, NodeKind, Point,
    PortConstraints, PortDirection, Rect, Side,
};
use super::size::{carried_size, size_graph_default, LabelBox, Sizes};
use super::style::{edge_style, node_style, port_style, EdgeStyle, NodeStyle, PortStyle};

/// The root element id. One editor instance hosts one graph at a time, so a
/// constant id lets the client address the root without reading the JSON.
pub const ROOT_ID: &str = "sysml-diagram";

/// Suffix appended to a node id to form its synthetic label child's id.
pub const LABEL_SUFFIX: &str = "-label";

/// sprotty element `type` for container/block-like IR nodes.
pub const TYPE_NODE: &str = "node";
/// sprotty element `type` for IR ports.
pub const TYPE_PORT: &str = "port";
/// sprotty element `type` for labels (IR label nodes and synthetic name labels).
pub const TYPE_LABEL: &str = "label";
/// sprotty element `type` for IR compartments.
pub const TYPE_COMPARTMENT: &str = "compartment";
/// sprotty element `type` for edges.
pub const TYPE_EDGE: &str = "edge";

/// The sprotty element `type` an IR node kind maps to.
pub fn sprotty_type(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Port => TYPE_PORT,
        NodeKind::Label => TYPE_LABEL,
        NodeKind::Compartment => TYPE_COMPARTMENT,
        _ => TYPE_NODE,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

/// ELK option ids derived from [`LayoutHints`] (design §6.2). Serialised under
/// the root's `layoutOptions`; Phase 2 hands them to `sprotty-elk` unchanged.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LayoutOptions {
    #[serde(rename = "elk.algorithm")]
    pub algorithm: &'static str,
    #[serde(rename = "elk.direction")]
    pub direction: &'static str,
    /// `INCLUDE_CHILDREN` for a compound (hierarchical) layout; omitted otherwise.
    #[serde(rename = "elk.hierarchyHandling", skip_serializing_if = "Option::is_none")]
    pub hierarchy_handling: Option<&'static str>,
    #[serde(rename = "elk.portConstraints")]
    pub port_constraints: &'static str,
    /// Edge kinds that point "up" the layering (supertype above subtype); the
    /// client reverses them for layering only. Not an ELK id — syscribe's own.
    #[serde(rename = "syscribe.reversedEdgeKinds")]
    pub reversed_edge_kinds: Vec<&'static str>,
    /// `DEPTH_FIRST` for the behaviour kinds (StateMachine, Action): the
    /// layering follows the flow from its initial node instead of the greedy
    /// cycle breaker's pick, so a cyclic machine still reads top-down.
    #[serde(rename = "elk.layered.cycleBreaking.strategy", skip_serializing_if = "Option::is_none")]
    pub cycle_breaking: Option<&'static str>,
}

impl LayoutOptions {
    /// The options of a graph: its kind's hints, plus the behaviour kinds'
    /// cycle-breaking strategy.
    pub fn for_graph(graph: &DiagramGraph) -> LayoutOptions {
        let mut o = LayoutOptions::from_hints(&graph.layout_hints);
        if matches!(graph.kind, super::ir::DiagramKind::StateMachine | super::ir::DiagramKind::Action) {
            o.cycle_breaking = Some("DEPTH_FIRST");
        }
        o
    }

    pub fn from_hints(hints: &LayoutHints) -> LayoutOptions {
        LayoutOptions {
            cycle_breaking: None,
            algorithm: match hints.algorithm {
                LayoutAlgorithm::Layered => "layered",
                LayoutAlgorithm::Fixed => "fixed",
            },
            direction: match hints.direction {
                LayoutDirection::Down => "DOWN",
                LayoutDirection::Right => "RIGHT",
            },
            hierarchy_handling: hints.hierarchical.then_some("INCLUDE_CHILDREN"),
            port_constraints: match hints.port_constraints {
                PortConstraints::Free => "FREE",
                PortConstraints::FixedSide => "FIXED_SIDE",
            },
            reversed_edge_kinds: hints.reversed_kinds.iter().map(EdgeKind::as_str).collect(),
        }
    }
}

/// A synthetic label of a node, port, compartment or edge.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SLabel {
    pub id: String,
    #[serde(rename = "type")]
    pub element_type: &'static str,
    pub text: String,
    /// The client's `LabelRole` (`name`, `stereotype`, `banner`, `line`,
    /// `keyword`, `edge`), which fixes the font.
    pub role: &'static str,
    /// The text box from the shared metrics (`REQ-TRS-VIS-017`).
    pub size: Size,
}

impl SLabel {
    fn of(l: &LabelBox, element_type: &'static str) -> SLabel {
        SLabel { id: l.id.clone(), element_type, text: l.text.clone(), role: l.role.as_str(), size: Size { width: l.w, height: l.h } }
    }
}

/// sprotty element `type` for an edge's labels.
pub const TYPE_EDGE_LABEL: &str = "label:edge";

/// A child of a node: its synthetic label, or a nested IR node.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum SNodeChild {
    Label(SLabel),
    Node(SNode),
}

/// The resolved style of an element, by its sprotty type: a [`NodeStyle`] on
/// a `node`, a [`PortStyle`] on a `port`; labels and compartments carry none.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum SStyle {
    Node(NodeStyle),
    Port(PortStyle),
}

/// One IR node as a sprotty element (`node`, `port`, `label` or `compartment`).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SNode {
    pub id: String,
    #[serde(rename = "type")]
    pub element_type_: &'static str,
    #[serde(rename = "ref")]
    pub element_ref: String,
    pub resolved: bool,
    /// The IR [`NodeKind`] spelling (`block`, `boundary`, `port`, …).
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stereotype: Option<String>,
    /// The display label.
    pub name: String,
    /// `label`-typed elements only: the text to draw (same as `name`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// `compartment`-typed elements only: one entry per line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines: Option<Vec<String>>,
    pub is_abstract: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<&'static str>,
    /// Always present on a `port` element (`null` when no side is known);
    /// omitted on everything else unless the IR carries one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<Option<&'static str>>,
    /// Present only for a pinned node; parent-relative when nested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
    /// Always present: the carried size (module doc).
    pub size: Size,
    /// `label`-typed elements only: the client's `LabelRole` (`free`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<&'static str>,
    /// `node` and `port` elements only (`REQ-TRS-VIS-012`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<SStyle>,
    /// `node` elements only, omitted when empty: applied-stereotype banners.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub banners: Vec<String>,
    pub children: Vec<SNodeChild>,
}

/// One IR edge as a sprotty element. Always a root child.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SEdge {
    pub id: String,
    #[serde(rename = "type")]
    pub element_type_: &'static str,
    pub source_id: String,
    pub target_id: String,
    /// The IR [`EdgeKind`] spelling.
    pub kind: &'static str,
    #[serde(rename = "ref", skip_serializing_if = "Option::is_none")]
    pub element_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Pinned routing, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing_points: Option<Vec<Position>>,
    /// The kind's notation (`REQ-TRS-VIS-012`).
    pub style: EdgeStyle,
    /// The `«keyword»` and label as sized `label:edge` children; omitted
    /// when the edge has neither.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<SLabel>,
}

/// A root child: a top-level node or an edge.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum SChild {
    Node(SNode),
    Edge(SEdge),
}

/// The root sprotty graph.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SGraph {
    pub id: &'static str,
    #[serde(rename = "type")]
    pub element_type_: &'static str,
    pub qualified_name: String,
    /// [`DiagramKind::as_str`](super::DiagramKind::as_str).
    pub diagram_kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    pub layout_options: LayoutOptions,
    /// Ids of pinned nodes, in declaration order.
    pub pinned: Vec<String>,
    /// Whether the content is generated from the model
    /// ([`DiagramGraph::derived`]): the client can tell a regenerated node set
    /// from an authored one without guessing from the ids.
    pub derived: bool,
    pub children: Vec<SChild>,
}

fn side_str(side: Side) -> &'static str {
    match side {
        Side::North => "north",
        Side::East => "east",
        Side::South => "south",
        Side::West => "west",
    }
}

/// A port's side: the IR's, else the conventional side for its direction
/// (inputs enter from the west, outputs leave east, bidirectional ports sit
/// south), else none.
pub fn port_side(node: &Node) -> Option<&'static str> {
    node.side.map(side_str).or(match node.direction {
        Some(PortDirection::In) => Some("west"),
        Some(PortDirection::Out) => Some("east"),
        Some(PortDirection::Inout) => Some("south"),
        None => None,
    })
}

fn position_of(pin: &Rect) -> Position {
    Position { x: pin.x, y: pin.y }
}

/// Whether `node` is drawn at the root: it has no parent, or names a parent
/// that is not in the graph (the manifest already drops those, but a
/// generator must not be able to make this writer lose a node).
fn is_root(graph: &DiagramGraph, node: &Node) -> bool {
    match node.parent.as_deref() {
        None => true,
        Some(p) => graph.node(p).is_none(),
    }
}

fn build_node(graph: &DiagramGraph, sizes: &Sizes, node: &Node, depth: usize) -> SNode {
    let element_type_ = sprotty_type(node.kind);
    let mut children = Vec::new();
    if let Some(sizing) = sizes.node(&node.id) {
        // Stereotype, banners, name (node/port); lines (compartment). A free
        // IR label inside a node is a child of its own below, not a label here.
        for l in sizing.labels.iter().filter(|l| l.role != super::size::LabelRole::Free) {
            children.push(SNodeChild::Label(SLabel::of(l, TYPE_LABEL)));
        }
    }
    // A `parent:` cycle cannot survive the manifest parser, but bound the
    // recursion by the node count anyway so a bad generator can't overflow.
    if depth < graph.nodes.len() {
        for child in graph.children_of(&node.id) {
            children.push(SNodeChild::Node(build_node(graph, sizes, child, depth + 1)));
        }
    }
    let carried = carried_size(node, sizes);
    SNode {
        id: node.id.clone(),
        element_type_,
        element_ref: node.element_ref.clone(),
        resolved: node.resolved,
        kind: node.kind.as_str(),
        element_type: node.element_type.clone(),
        stereotype: node.stereotype.clone(),
        name: node.label.clone(),
        text: (element_type_ == TYPE_LABEL).then(|| node.label.clone()),
        lines: (element_type_ == TYPE_COMPARTMENT).then(|| node.lines.clone()),
        is_abstract: node.is_abstract,
        direction: node.direction.map(|d| d.as_str()),
        side: if element_type_ == TYPE_PORT { Some(port_side(node)) } else { node.side.map(side_str).map(Some) },
        position: node.pin.as_ref().map(position_of),
        size: Size { width: carried.w, height: carried.h },
        role: (element_type_ == TYPE_LABEL).then_some("free"),
        style: match element_type_ {
            TYPE_NODE => Some(SStyle::Node(node_style(node))),
            TYPE_PORT => Some(SStyle::Port(port_style(node.direction))),
            _ => None,
        },
        banners: if element_type_ == TYPE_NODE { node.banners.clone() } else { Vec::new() },
        children,
    }
}

fn build_edge(edge: &Edge, sizes: &Sizes) -> SEdge {
    SEdge {
        children: sizes.edges.get(&edge.id).map(|ls| ls.iter().map(|l| SLabel::of(l, TYPE_EDGE_LABEL)).collect()).unwrap_or_default(),
        id: edge.id.clone(),
        element_type_: TYPE_EDGE,
        source_id: edge.source.clone(),
        target_id: edge.target.clone(),
        kind: edge.kind.as_str(),
        element_ref: edge.element_ref.clone(),
        label: edge.label.clone(),
        routing_points: edge
            .waypoints
            .as_ref()
            .map(|pts| pts.iter().map(|Point { x, y }| Position { x: *x, y: *y }).collect()),
        style: edge_style(edge.kind),
    }
}

/// Serialise the IR as a nested sprotty `SGraph` (see the module doc for the
/// contract), sizing every element with the process-wide diagram metrics.
/// Serialise the result with `serde_json` or wrap it in axum's `Json`;
/// [`to_sgraph_json`] does the former.
pub fn to_sgraph(graph: &DiagramGraph) -> SGraph {
    to_sgraph_with(graph, &size_graph_default(graph))
}

/// [`to_sgraph`] with precomputed [`Sizes`].
pub fn to_sgraph_with(graph: &DiagramGraph, sizes: &Sizes) -> SGraph {
    let mut children: Vec<SChild> = graph
        .nodes
        .iter()
        .filter(|n| is_root(graph, n))
        .map(|n| SChild::Node(build_node(graph, sizes, n, 0)))
        .collect();
    children.extend(graph.edges.iter().map(|e| SChild::Edge(build_edge(e, sizes))));
    SGraph {
        id: ROOT_ID,
        element_type_: "graph",
        qualified_name: graph.qualified_name.clone(),
        diagram_kind: graph.kind.as_str(),
        subject: graph.subject.clone(),
        layout_options: LayoutOptions::for_graph(graph),
        pinned: graph.pinned_ids().into_iter().map(str::to_string).collect(),
        derived: graph.derived,
        children,
    }
}

/// [`to_sgraph`] as a `serde_json::Value`.
pub fn to_sgraph_json(graph: &DiagramGraph) -> serde_json::Value {
    serde_json::to_value(to_sgraph(graph)).expect("SGraph serialises")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vis::ir::{DiagramKind, PortDirection};
    use serde_json::{json, Value};

    fn node(id: &str, kind: NodeKind, parent: Option<&str>) -> Node {
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
            pin: None,
            banners: vec![],
        }
    }

    /// boundary `sys` ⊃ block `engine` ⊃ port `pout`; block `motor` pinned
    /// inside `sys` with a size; one flow edge port → motor with waypoints.
    fn ibd() -> DiagramGraph {
        let mut g = DiagramGraph::empty(DiagramKind::Ibd, "Diagrams::D", "D", Some("Sys"));
        g.nodes.push(node("sys", NodeKind::Boundary, None));
        g.nodes.push(node("engine", NodeKind::Block, Some("sys")));
        let mut pout = node("pout", NodeKind::Port, Some("engine"));
        pout.direction = Some(PortDirection::Out);
        pout.side = Some(Side::East);
        pout.stereotype = Some("port".into());
        g.nodes.push(pout);
        let mut motor = node("motor", NodeKind::Block, Some("sys"));
        motor.pin = Some(Rect { x: 220.0, y: 40.0, w: Some(150.0), h: Some(60.0) });
        motor.element_type = Some("PartDef".into());
        motor.stereotype = Some("part def".into());
        motor.is_abstract = true;
        g.nodes.push(motor);
        g.edges.push(Edge {
            id: "e1".into(),
            element_ref: Some("Sys".into()),
            source: "pout".into(),
            target: "motor".into(),
            kind: EdgeKind::Flow,
            label: Some("power".into()),
            waypoints: Some(vec![Point { x: 1.0, y: 2.0 }]),
        });
        g
    }

    fn child_ids(v: &Value) -> Vec<&str> {
        v["children"].as_array().unwrap().iter().map(|c| c["id"].as_str().unwrap()).collect()
    }

    #[test]
    fn nodes_nest_by_parent_and_ports_sit_inside_their_block() {
        let j = to_sgraph_json(&ibd());
        assert_eq!(j["id"], "sysml-diagram");
        assert_eq!(j["type"], "graph");
        assert_eq!(j["qualifiedName"], "Diagrams::D");
        assert_eq!(j["diagramKind"], "IBD");
        assert_eq!(j["subject"], "Sys");
        // Root: the boundary, then the edge — nothing else leaks to the top.
        assert_eq!(child_ids(&j), vec!["sys", "e1"]);
        let sys = &j["children"][0];
        assert_eq!(sys["type"], "node");
        assert_eq!(sys["kind"], "boundary");
        assert_eq!(child_ids(sys), vec!["sys-label", "engine", "motor"]);
        let engine = &sys["children"][1];
        assert_eq!(child_ids(engine), vec!["engine-label", "pout"]);
        let pout = &engine["children"][1];
        assert_eq!(pout["type"], "port");
        assert_eq!(pout["kind"], "port");
        assert_eq!(pout["direction"], "out");
        assert_eq!(pout["side"], "east");
        assert_eq!(pout["stereotype"], "port");
        // A port carries only its name label, sized (`REQ-TRS-VIS-017`).
        let kids = pout["children"].as_array().unwrap();
        assert_eq!(kids.len(), 1);
        assert_eq!(kids[0]["id"], "pout-label");
        assert_eq!(kids[0]["type"], "label");
        assert_eq!(kids[0]["text"], "POUT");
        assert_eq!(kids[0]["role"], "name");
        assert!(kids[0]["size"]["width"].as_f64().unwrap() > 0.0 && kids[0]["size"]["height"].as_f64().unwrap() > 0.0);
        assert_eq!(pout["size"], json!({ "width": 12.0, "height": 12.0 }));
        // A node with a stereotype carries it before its name label.
        let motor = &sys["children"][2];
        assert_eq!(child_ids(motor), vec!["motor-stereotype", "motor-label"]);
        assert_eq!(motor["children"][0]["role"], "stereotype");
        assert_eq!(motor["children"][0]["text"], "«part def»");
        assert_eq!(motor["children"][1]["role"], "name");
    }

    #[test]
    fn nodes_ports_and_edges_carry_their_resolved_style() {
        let mut g = ibd();
        g.node_mut("motor").banners = vec!["Safety".into(), "Rationale".into()];
        let j = to_sgraph_json(&g);
        let sys = &j["children"][0];
        // An untyped boundary falls back to the kind's colours; no banners key.
        assert_eq!(
            sys["style"],
            json!({ "fill": "#f8f9fb", "stroke": "#3a3a4a", "headerFill": null, "text": "#222", "dashed": false })
        );
        assert!(sys.get("banners").is_none(), "empty banners are omitted");
        let motor = &sys["children"][2];
        assert_eq!(motor["style"]["fill"], "#f5f5fa");
        assert_eq!(motor["style"]["stroke"], "#3a3a4a");
        assert_eq!(motor["banners"], json!(["Safety", "Rationale"]));
        // A port carries the port style, never the node one, and no banners.
        let pout = &sys["children"][1]["children"][1];
        assert_eq!(pout["style"], json!({ "fill": "#333", "stroke": "#1f497d", "glyph": "out" }));
        assert!(pout.get("banners").is_none());
        // The label child and a compartment carry no style.
        assert!(sys["children"][0].get("style").is_none());
        let mut g2 = DiagramGraph::empty(DiagramKind::Bdd, "D", "D", None);
        g2.nodes.push(node("a", NodeKind::Block, None));
        g2.nodes.push(node("a-c", NodeKind::Compartment, Some("a")));
        let j2 = to_sgraph_json(&g2);
        assert!(j2["children"][0]["children"][1].get("style").is_none());
        // An unresolved node is dashed.
        let mut g3 = ibd();
        g3.node_mut("engine").resolved = false;
        assert_eq!(to_sgraph_json(&g3)["children"][0]["children"][1]["style"]["dashed"], true);
    }

    #[test]
    fn port_side_is_the_irs_else_derived_from_direction_else_null() {
        let mut g = ibd();
        let j = to_sgraph_json(&g);
        assert_eq!(j["children"][0]["children"][1]["children"][1]["side"], "east", "explicit side wins");
        g.node_mut("pout").side = None;
        assert_eq!(to_sgraph_json(&g)["children"][0]["children"][1]["children"][1]["side"], "east", "out → east");
        g.node_mut("pout").direction = Some(PortDirection::In);
        assert_eq!(to_sgraph_json(&g)["children"][0]["children"][1]["children"][1]["side"], "west");
        g.node_mut("pout").direction = Some(PortDirection::Inout);
        let j = to_sgraph_json(&g);
        let pout = &j["children"][0]["children"][1]["children"][1];
        assert_eq!(pout["side"], "south");
        assert_eq!(pout["style"]["glyph"], "inout");
        assert_eq!(pout["style"]["fill"], "#fff");
        g.node_mut("pout").direction = None;
        let j = to_sgraph_json(&g);
        let pout = &j["children"][0]["children"][1]["children"][1];
        assert_eq!(pout["side"], Value::Null, "a port always carries `side`, null when unknown");
        assert!(pout.as_object().unwrap().contains_key("side"));
        assert_eq!(pout["style"]["glyph"], "none");
        // A non-port without a side omits the key entirely.
        assert!(j["children"][0].get("side").is_none());
    }

    #[test]
    fn position_only_for_pinned_nodes_and_size_on_every_element() {
        let j = to_sgraph_json(&ibd());
        let sys = &j["children"][0];
        assert!(sys.get("position").is_none(), "unpinned boundary has no position");
        // Every node carries a size: the metrics box for an unpinned node …
        let sys_size = &sys["size"];
        assert!(sys_size["width"].as_f64().unwrap() > 0.0 && sys_size["height"].as_f64().unwrap() > 0.0);
        let engine = &sys["children"][1];
        assert!(engine.get("position").is_none());
        let engine_w = engine["size"]["width"].as_f64().unwrap();
        assert!(engine_w >= 120.0, "a leaf is at least the client's minimum: {engine_w}");
        // … and the pin's own `w`/`h` for a node that records both.
        let motor = &sys["children"][2];
        assert_eq!(motor["position"], json!({ "x": 220.0, "y": 40.0 }));
        assert_eq!(motor["size"], json!({ "width": 150.0, "height": 60.0 }));
        assert_eq!(motor["isAbstract"], true);
        assert_eq!(motor["elementType"], "PartDef");
        assert_eq!(motor["stereotype"], "part def");
        assert_eq!(motor["name"], "MOTOR");
        assert_eq!(motor["ref"], "Sys::motor");
        assert_eq!(motor["resolved"], true);
        assert_eq!(j["pinned"], json!(["motor"]));

        // A pin with only x/y (or w without h): position, and the metrics size.
        let mut g = ibd();
        g.node_mut("engine").pin = Some(Rect { x: 1.0, y: 2.0, w: Some(10.0), h: None });
        let j = to_sgraph_json(&g);
        let engine = &j["children"][0]["children"][1];
        assert_eq!(engine["position"], json!({ "x": 1.0, "y": 2.0 }));
        assert_eq!(engine["size"]["width"], engine_w, "w without h is not a carried size");
        assert_eq!(j["pinned"], json!(["engine", "motor"]));
    }

    #[test]
    fn edges_stay_top_level_with_kind_ref_label_and_routing_points() {
        let j = to_sgraph_json(&ibd());
        let mut e = j["children"][1].clone();
        // The label is also a sized `label:edge` child (`REQ-TRS-VIS-017`).
        let kids = e.as_object_mut().unwrap().remove("children").expect("edge label children");
        assert_eq!(kids[0]["id"], "e1-label");
        assert_eq!(kids[0]["type"], "label:edge");
        assert_eq!(kids[0]["role"], "edge");
        assert_eq!(kids[0]["text"], "power");
        assert!(kids[0]["size"]["width"].as_f64().unwrap() > 0.0);
        assert_eq!(
            e,
            json!({
                "id": "e1", "type": "edge", "sourceId": "pout", "targetId": "motor",
                "kind": "flow", "ref": "Sys", "label": "power",
                "routingPoints": [{ "x": 1.0, "y": 2.0 }],
                "style": {
                    "stroke": "#1f497d", "dash": null, "width": 1.4,
                    "arrowTarget": "filled", "arrowSource": "none", "keyword": null
                }
            })
        );
        // Every edge carries its kind's notation; the diamond of a composition
        // sits at the source (the owner).
        let mut g = ibd();
        g.edges[0].kind = EdgeKind::Composition;
        let s = &to_sgraph_json(&g)["children"][1]["style"];
        assert_eq!(s["arrowSource"], "filledDiamond");
        assert_eq!(s["arrowTarget"], "none");
        g.edges[0].kind = EdgeKind::Binding;
        let e = &to_sgraph_json(&g)["children"][1];
        let s = &e["style"];
        assert_eq!(s["dash"], "4,4");
        assert_eq!(s["keyword"], "=");
        assert_eq!(child_ids(e), vec!["e1-keyword", "e1-label"], "keyword before label");
        assert_eq!(e["children"][0]["role"], "keyword");
        assert_eq!(e["children"][0]["text"], "=");
        g.edges[0].kind = EdgeKind::Verify;
        let e = &to_sgraph_json(&g)["children"][1];
        let s = &e["style"];
        assert_eq!(s["stroke"], "#3a6ea5");
        assert_eq!(s["keyword"], "«verify»");
        assert_eq!(s["arrowTarget"], "open");
        assert_eq!(e["children"][0]["text"], "«verify»");
        g.edges[0].kind = EdgeKind::Connection;
        g.edges[0].label = None;
        assert!(to_sgraph_json(&g)["children"][1].get("children").is_none(), "no keyword, no label: no children key");
        // The nested nodes carry no edges.
        let walk = |v: &Value| v["children"].as_array().unwrap().iter().all(|c| c["type"] != "edge");
        assert!(walk(&j["children"][0]));
        assert!(walk(&j["children"][0]["children"][1]));
    }

    #[test]
    fn layout_options_follow_the_kind() {
        let j = to_sgraph_json(&ibd());
        assert_eq!(
            j["layoutOptions"],
            json!({
                "elk.algorithm": "layered",
                "elk.direction": "RIGHT",
                "elk.hierarchyHandling": "INCLUDE_CHILDREN",
                "elk.portConstraints": "FIXED_SIDE",
                "syscribe.reversedEdgeKinds": []
            })
        );
        let bdd = DiagramGraph::empty(DiagramKind::Bdd, "D", "D", None);
        let j = to_sgraph_json(&bdd);
        assert_eq!(
            j["layoutOptions"],
            json!({
                "elk.algorithm": "layered",
                "elk.direction": "DOWN",
                "elk.portConstraints": "FREE",
                "syscribe.reversedEdgeKinds": ["inheritance"]
            })
        );
        assert!(j.get("subject").is_none());
        assert_eq!(j["children"], json!([]));
        assert_eq!(j["pinned"], json!([]));
        assert_eq!(j["derived"], json!(false), "a manifest/empty graph is not derived");
        let mut derived = DiagramGraph::empty(DiagramKind::Bdd, "D", "D", Some("Sys"));
        derived.derived = true;
        assert_eq!(to_sgraph_json(&derived)["derived"], json!(true));
        let seq = DiagramGraph::empty(DiagramKind::Sequence, "D", "D", None);
        assert_eq!(to_sgraph_json(&seq)["layoutOptions"]["elk.algorithm"], "fixed");
    }

    #[test]
    fn labels_and_compartments_map_to_their_own_types() {
        let mut g = DiagramGraph::empty(DiagramKind::Bdd, "D", "D", None);
        g.nodes.push(node("a", NodeKind::Block, None));
        let mut comp = node("a-attrs", NodeKind::Compartment, Some("a"));
        comp.lines = vec!["mass : Real [kg]".into(), "powerOut : PowerPort".into()];
        g.nodes.push(comp);
        g.nodes.push(node("free", NodeKind::Label, None));
        let j = to_sgraph_json(&g);
        assert_eq!(child_ids(&j), vec!["a", "free"]);
        let comp = &j["children"][0]["children"][1];
        assert_eq!(comp["type"], "compartment");
        assert_eq!(comp["kind"], "compartment");
        assert_eq!(comp["lines"], json!(["mass : Real [kg]", "powerOut : PowerPort"]));
        // One sized `line` label per line (`REQ-TRS-VIS-017`).
        assert_eq!(child_ids(comp), vec!["a-attrs-line-0", "a-attrs-line-1"]);
        assert_eq!(comp["children"][1]["role"], "line");
        assert_eq!(comp["children"][1]["text"], "powerOut : PowerPort");
        assert!(comp["children"][1]["size"]["height"].as_f64().unwrap() > 0.0);
        assert!(comp["size"]["width"].as_f64().unwrap() > 0.0);
        assert!(comp.get("text").is_none());
        let free = &j["children"][1];
        assert_eq!(free["type"], "label");
        assert_eq!(free["text"], "FREE");
        assert_eq!(free["role"], "free");
        assert!(free["size"]["width"].as_f64().unwrap() > 0.0);
        assert!(free.get("lines").is_none());
        assert_eq!(free["children"], json!([]));
    }

    #[test]
    fn output_is_in_declaration_order_and_byte_stable() {
        let mut g = DiagramGraph::empty(DiagramKind::Bdd, "D", "D", None);
        for id in ["zeta", "alpha", "mid"] {
            g.nodes.push(node(id, NodeKind::Block, None));
        }
        for (i, (s, t)) in [("zeta", "alpha"), ("mid", "zeta")].iter().enumerate() {
            g.edges.push(Edge {
                id: format!("e{i}"),
                element_ref: None,
                source: s.to_string(),
                target: t.to_string(),
                kind: EdgeKind::Inheritance,
                label: None,
                waypoints: None,
            });
        }
        let a = serde_json::to_string(&to_sgraph(&g)).unwrap();
        let b = serde_json::to_string(&to_sgraph(&g)).unwrap();
        assert_eq!(a, b);
        assert_eq!(child_ids(&to_sgraph_json(&g)), vec!["zeta", "alpha", "mid", "e0", "e1"]);
        // Field order is the struct's, not alphabetical.
        assert!(a.starts_with(r#"{"id":"sysml-diagram","type":"graph","qualifiedName":"D","diagramKind":"BDD","layoutOptions":{"elk.algorithm":"layered""#), "{a}");
    }

    #[test]
    fn a_node_whose_parent_is_missing_is_drawn_at_the_root() {
        let mut g = DiagramGraph::empty(DiagramKind::Custom, "D", "D", None);
        g.nodes.push(node("orphan", NodeKind::Block, Some("ghost")));
        assert_eq!(child_ids(&to_sgraph_json(&g)), vec!["orphan"]);
    }

    impl DiagramGraph {
        fn node_mut(&mut self, id: &str) -> &mut Node {
            self.nodes.iter_mut().find(|n| n.id == id).unwrap()
        }
    }
}
