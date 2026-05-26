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
pub struct ChildSummary {
    pub qualified_name: String,
    pub name: Option<String>,
    pub element_type: Option<String>,
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
) -> Result<Json<serde_json::Value>, StatusCode> {
    let qname_raw = params.get("of").ok_or(StatusCode::BAD_REQUEST)?.clone();
    let qname_norm = qname_raw.replace('/', "::");

    let store = state.read().await;

    let element = store
        .resolver
        .get(&store.elements, &qname_norm)
        .ok_or(StatusCode::NOT_FOUND)?;

    let connections = serde_json::json!({
        "qualifiedName": element.qualified_name,
        "connections": element.frontmatter.connections,
        "flowConnections": element.frontmatter.flow_connections,
        "bindingConnections": element.frontmatter.binding_connections,
        "successionConnections": element.frontmatter.succession_connections,
        "exhibitsStates": element.frontmatter.exhibits_states,
    });

    Ok(Json(connections))
}
