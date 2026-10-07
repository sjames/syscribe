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
//! Each `node`/`port` element also carries a synthetic label child
//! `{ id: "<id>-label", type: "label", text }` as its *first* child, so the
//! client's hidden render pass can measure the text in Phase 2
//! (`REQ-TRS-VIS-007`). An IR [`NodeKind::Label`] node is itself a `label`
//! element and carries its `text` directly; a [`NodeKind::Compartment`] node is
//! a `compartment` element carrying `lines` and gets no label child.
//!
//! ## Geometry
//!
//! `position` is present **only** when the IR node is pinned
//! ([`Node::pin`]), and `size` only when that pin carries both `w` and `h`.
//! A pinned position of a *nested* node is relative to its parent's origin —
//! sprotty's own local-coordinate convention, and exactly what
//! `PATCH /api/diagrams/layout` writes back from a drag of a nested node, so
//! `layout:` round-trips without conversion. Unpinned nodes carry no geometry
//! at all; in Phase 0 the client places them with a trivial deterministic
//! arrangement and default sizes by kind (`frontend/src/layout-shim.ts`), and
//! Phase 2 replaces that shim with measured sizes and ELK. Keeping the shim
//! out of this writer keeps the contract honest: a `position` in the JSON
//! always means "a human pinned this".
//!
//! `pinned` lists the ids of pinned nodes in declaration order, so the client
//! can tell "fixed by a human" from "placed by the layout engine".
//!
//! ## Ordering
//!
//! Output order is the IR's declaration order throughout (nodes, their
//! children, then edges) — never hash order — so the JSON is stable across
//! runs and diffable in tests.

use serde::Serialize;

use super::ir::{
    DiagramGraph, Edge, EdgeKind, LayoutAlgorithm, LayoutDirection, LayoutHints, Node, NodeKind, Point,
    PortConstraints, Rect, Side,
};

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
}

impl LayoutOptions {
    pub fn from_hints(hints: &LayoutHints) -> LayoutOptions {
        LayoutOptions {
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

/// The synthetic name label of a node or port.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SLabel {
    pub id: String,
    #[serde(rename = "type")]
    pub element_type: &'static str,
    pub text: String,
}

/// A child of a node: its synthetic label, or a nested IR node.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum SNodeChild {
    Label(SLabel),
    Node(SNode),
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<&'static str>,
    /// Present only for a pinned node; parent-relative when nested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
    /// Present only when the pin carries both `w` and `h`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<Size>,
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

fn position_of(pin: &Rect) -> Position {
    Position { x: pin.x, y: pin.y }
}

fn size_of(pin: &Rect) -> Option<Size> {
    match (pin.w, pin.h) {
        (Some(width), Some(height)) => Some(Size { width, height }),
        _ => None,
    }
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

fn build_node(graph: &DiagramGraph, node: &Node, depth: usize) -> SNode {
    let element_type_ = sprotty_type(node.kind);
    let mut children = Vec::new();
    if matches!(element_type_, TYPE_NODE | TYPE_PORT) {
        children.push(SNodeChild::Label(SLabel {
            id: format!("{}{}", node.id, LABEL_SUFFIX),
            element_type: TYPE_LABEL,
            text: node.label.clone(),
        }));
    }
    // A `parent:` cycle cannot survive the manifest parser, but bound the
    // recursion by the node count anyway so a bad generator can't overflow.
    if depth < graph.nodes.len() {
        for child in graph.children_of(&node.id) {
            children.push(SNodeChild::Node(build_node(graph, child, depth + 1)));
        }
    }
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
        side: node.side.map(side_str),
        position: node.pin.as_ref().map(position_of),
        size: node.pin.as_ref().and_then(size_of),
        children,
    }
}

fn build_edge(edge: &Edge) -> SEdge {
    SEdge {
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
    }
}

/// Serialise the IR as a nested sprotty `SGraph` (see the module doc for the
/// contract). Serialise the result with `serde_json` or wrap it in axum's
/// `Json`; [`to_sgraph_json`] does the former.
pub fn to_sgraph(graph: &DiagramGraph) -> SGraph {
    let mut children: Vec<SChild> = graph
        .nodes
        .iter()
        .filter(|n| is_root(graph, n))
        .map(|n| SChild::Node(build_node(graph, n, 0)))
        .collect();
    children.extend(graph.edges.iter().map(|e| SChild::Edge(build_edge(e))));
    SGraph {
        id: ROOT_ID,
        element_type_: "graph",
        qualified_name: graph.qualified_name.clone(),
        diagram_kind: graph.kind.as_str(),
        subject: graph.subject.clone(),
        layout_options: LayoutOptions::from_hints(&graph.layout_hints),
        pinned: graph.pinned_ids().into_iter().map(str::to_string).collect(),
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
        assert_eq!(pout["children"], json!([{ "id": "pout-label", "type": "label", "text": "POUT" }]));
    }

    #[test]
    fn position_only_for_pinned_nodes_and_size_only_with_w_and_h() {
        let j = to_sgraph_json(&ibd());
        let sys = &j["children"][0];
        assert!(sys.get("position").is_none(), "unpinned boundary has no position");
        assert!(sys.get("size").is_none());
        let engine = &sys["children"][1];
        assert!(engine.get("position").is_none());
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

        // A pin with only x/y: position, no size.
        let mut g = ibd();
        g.node_mut("engine").pin = Some(Rect { x: 1.0, y: 2.0, w: Some(10.0), h: None });
        let j = to_sgraph_json(&g);
        let engine = &j["children"][0]["children"][1];
        assert_eq!(engine["position"], json!({ "x": 1.0, "y": 2.0 }));
        assert!(engine.get("size").is_none(), "w without h is not a size");
        assert_eq!(j["pinned"], json!(["engine", "motor"]));
    }

    #[test]
    fn edges_stay_top_level_with_kind_ref_label_and_routing_points() {
        let j = to_sgraph_json(&ibd());
        let e = &j["children"][1];
        assert_eq!(
            *e,
            json!({
                "id": "e1", "type": "edge", "sourceId": "pout", "targetId": "motor",
                "kind": "flow", "ref": "Sys", "label": "power",
                "routingPoints": [{ "x": 1.0, "y": 2.0 }]
            })
        );
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
        assert_eq!(comp["children"], json!([]), "a compartment gets no label child");
        assert!(comp.get("text").is_none());
        let free = &j["children"][1];
        assert_eq!(free["type"], "label");
        assert_eq!(free["text"], "FREE");
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
