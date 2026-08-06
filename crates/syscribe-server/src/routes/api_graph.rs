use std::collections::HashMap;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use syscribe_model::graph::children_of;
use crate::state::SharedState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChildSummary {
    pub qualified_name: String,
    pub name: Option<String>,
    pub element_type: Option<String>,
}

/// `GET /api/connections`'s response — replaces the previous hand-built
/// `serde_json::json!{}` literal so this route uses the same typed-struct +
/// blanket-derive convention as every other JSON route. Field types mirror
/// `RawFrontmatter`'s untyped connection fields (`syscribe_model::element`)
/// verbatim; `serde_yaml::Value` serializes to JSON structurally identically
/// to how the retired `json!{}` literal serialized it (via `serde_json::
/// to_value` on the same `Serialize` impl), so this is not a wire change.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionsResponse {
    pub qualified_name: String,
    pub connections: Option<Vec<serde_yaml::Value>>,
    pub flow_connections: Option<Vec<serde_yaml::Value>>,
    pub binding_connections: Option<Vec<serde_yaml::Value>>,
    pub succession_connections: Option<Vec<serde_yaml::Value>>,
    pub exhibits_states: Option<Vec<String>>,
}

/// GET /api/children?of=<qualifiedName>
pub async fn get_children(
    State(state): State<SharedState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ChildSummary>>, StatusCode> {
    let qname_raw = params.get("of").ok_or(StatusCode::BAD_REQUEST)?.clone();
    // Allow callers to use either "/" or "::" as separator
    let qname_norm = qname_raw.replace('/', "::");

    let store = state.read().await;

    if !store.node_idx.contains_key(&qname_norm) {
        return Err(StatusCode::NOT_FOUND);
    }

    let children = children_of(&store.graph, &store.node_idx, &qname_norm);
    let result: Vec<ChildSummary> = children
        .iter()
        .filter_map(|&cqn| {
            store.resolver.get(&store.elements, cqn).map(|e| ChildSummary {
                qualified_name: e.qualified_name.clone(),
                name: e.frontmatter.name.clone(),
                element_type: e
                    .frontmatter
                    .element_type
                    .as_ref()
                    .map(|t| format!("{:?}", t)),
            })
        })
        .collect();

    Ok(Json(result))
}

/// GET /api/connections?of=<qualifiedName>
/// Returns the raw connection frontmatter fields as JSON.
pub async fn get_connections(
    State(state): State<SharedState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ConnectionsResponse>, StatusCode> {
    let qname_raw = params.get("of").ok_or(StatusCode::BAD_REQUEST)?.clone();
    let qname_norm = qname_raw.replace('/', "::");

    let store = state.read().await;

    let element = store
        .resolver
        .get(&store.elements, &qname_norm)
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(ConnectionsResponse {
        qualified_name: element.qualified_name.clone(),
        connections: element.frontmatter.connections.clone(),
        flow_connections: element.frontmatter.flow_connections.clone(),
        binding_connections: element.frontmatter.binding_connections.clone(),
        succession_connections: element.frontmatter.succession_connections.clone(),
        exhibits_states: element.frontmatter.exhibits_states.clone(),
    }))
}
