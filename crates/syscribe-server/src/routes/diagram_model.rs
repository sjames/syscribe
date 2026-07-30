//! `GET /api/diagrams/model/{*qname}` — JSON diagram-model endpoint for the
//! sprotty client (`ADR-SYS-DE-001`, `REQ-TRS-DE-004`). Purely additive: does
//! not change `render_diagram`, the Mermaid path, or any existing route.
//!
//! Parses the same `shapes:`/`edges:`/`layout:` frontmatter fields that
//! `syscribe_model::renderer::render_diagram` reads, using the shared
//! `syscribe_model::diagram` parse helpers/types (`DiagramShape`/`DiagramEdge`/
//! `ShapeLayout`/`parse_shapes`/`parse_edges`/`parse_layout`/`default_size`) —
//! the same source of truth `renderer.rs` itself now consumes.
//!
//! The emitted JSON is a flat sprotty-`SGraph`-shaped tree: a root
//! `{id, type: "graph", children: [...]}` whose children are either
//! `{id, type: "node", ref, kind, name, position, size}` (one per `shapes:`
//! entry) or `{id, type: "edge", sourceId, targetId, kind, ref?}` (one per
//! `edges:` entry) — matching `sprotty-protocol`'s `SNode`/`SEdge` schema
//! fields (`position`/`size`/`sourceId`/`targetId`) so the client can feed the
//! response straight into `LocalModelSource.setModel(...)`.
//!
//! ## Route shape note
//!
//! The natural REST path would be `/api/diagrams/{*qname}/model`, but a
//! catch-all path segment (`{*qname}`) must be the last component of an axum
//! route — the same constraint `routes::mutate`'s module doc documents for
//! why connection add/remove live at `/api/connections` instead of
//! `/api/elements/{*qname}/connections`. So this route is
//! `/api/diagrams/model/{*qname}`, a sibling of the existing
//! `/api/diagrams/layout/{*qname}` PATCH route (different literal prefix,
//! same catch-all-at-the-end shape — no routing conflict).

use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;

use syscribe_model::diagram::{default_size, parse_edges, parse_layout, parse_shapes, DEFAULT_DIAGRAM_KIND};
use syscribe_model::element::RawElement;

use crate::state::SharedState;

// ---------------------------------------------------------------------------
// Response JSON (sprotty SGraph-shaped)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    x: f64,
    y: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Size {
    width: f64,
    height: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeModel {
    id: String,
    #[serde(rename = "type")]
    node_type: &'static str,
    #[serde(rename = "ref")]
    element_ref: String,
    kind: String,
    name: String,
    is_abstract: bool,
    position: Position,
    size: Size,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EdgeModel {
    id: String,
    #[serde(rename = "type")]
    edge_type: &'static str,
    #[serde(rename = "ref")]
    element_ref: Option<String>,
    source_id: String,
    target_id: String,
    kind: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ChildModel {
    Node(NodeModel),
    Edge(EdgeModel),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagramModel {
    id: &'static str,
    #[serde(rename = "type")]
    root_type: &'static str,
    qualified_name: String,
    diagram_kind: String,
    /// The diagram's `subject:` field, if set — the natural "owning element"
    /// for a connect-edge gesture's `connections:` mutation (the diagram
    /// itself typically has no `connections:` list of its own).
    subject: Option<String>,
    children: Vec<ChildModel>,
}

/// `GET /api/diagrams/model/{*qname}` — see module doc comment.
///
/// `404` if `qname` doesn't resolve to a `Diagram` element. A `Diagram` with
/// no `shapes:`/`layout:` at all still returns `200` with empty `children`
/// (the client needs a valid, mountable — if empty — graph, unlike
/// `render_diagram`'s `None`-on-no-layout which the HTML path renders as an
/// explicit "no layout" message instead).
pub async fn get_diagram_model(
    State(state): State<SharedState>,
    Path(qname): Path<String>,
) -> Result<Json<DiagramModel>, StatusCode> {
    let store = state.read().await;
    let qname_norm = qname.replace('/', "::");
    let element = store
        .elements
        .iter()
        .find(|e| e.qualified_name == qname_norm)
        .ok_or(StatusCode::NOT_FOUND)?;

    let is_diagram = element
        .frontmatter
        .element_type
        .as_ref()
        .map(|t| format!("{:?}", t) == "Diagram")
        .unwrap_or(false);
    if !is_diagram {
        return Err(StatusCode::NOT_FOUND);
    }

    let diagram_kind = element
        .frontmatter
        .diagram_kind
        .clone()
        .unwrap_or_else(|| DEFAULT_DIAGRAM_KIND.to_string());

    let shapes = parse_shapes(element.frontmatter.shapes.as_ref());
    let edges = parse_edges(element.frontmatter.edges.as_ref());
    let layout = parse_layout(element.frontmatter.layout.as_ref());

    let elem_by_qname: HashMap<&str, &RawElement> = store
        .elements
        .iter()
        .map(|e| (e.qualified_name.as_str(), e))
        .collect();

    let mut children = Vec::with_capacity(shapes.len() + edges.len());

    let mut shape_ids: Vec<&String> = shapes.keys().collect();
    shape_ids.sort();
    for shape_id in shape_ids {
        let shape = &shapes[shape_id];
        let (dw, dh) = default_size(&shape.kind);
        let (x, y, w, h) = match layout.get(shape_id.as_str()) {
            Some(l) => (l.x, l.y, l.w.unwrap_or(dw), l.h.unwrap_or(dh)),
            None => (0.0, 0.0, dw, dh),
        };
        let (name, is_abstract) = match elem_by_qname.get(shape.element_ref.as_str()) {
            Some(e) => (
                e.frontmatter.name.clone().unwrap_or_else(|| {
                    e.qualified_name
                        .split("::")
                        .last()
                        .unwrap_or(&e.qualified_name)
                        .to_string()
                }),
                e.frontmatter.is_abstract.unwrap_or(false),
            ),
            None => (
                shape
                    .element_ref
                    .split("::")
                    .last()
                    .unwrap_or(&shape.element_ref)
                    .to_string(),
                false,
            ),
        };
        children.push(ChildModel::Node(NodeModel {
            id: shape_id.clone(),
            node_type: "node",
            element_ref: shape.element_ref.clone(),
            kind: shape.kind.clone(),
            name,
            is_abstract,
            position: Position { x, y },
            size: Size { width: w, height: h },
        }));
    }

    let mut edge_ids: Vec<&String> = edges.keys().collect();
    edge_ids.sort();
    for edge_id in edge_ids {
        let edge = &edges[edge_id];
        children.push(ChildModel::Edge(EdgeModel {
            id: edge_id.clone(),
            edge_type: "edge",
            element_ref: edge.element_ref.clone(),
            source_id: edge.source.clone(),
            target_id: edge.target.clone(),
            kind: edge.kind.clone(),
        }));
    }

    Ok(Json(DiagramModel {
        id: "sysml-diagram",
        root_type: "graph",
        qualified_name: element.qualified_name.clone(),
        diagram_kind,
        subject: element.frontmatter.subject.clone(),
        children,
    }))
}
