use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use syscribe_model::element::RawElement;
use crate::state::SharedState;

#[derive(Deserialize)]
pub struct ElementsQuery {
    #[serde(rename = "type")]
    pub type_filter: Option<String>,
}

/// Summary view for list responses.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementSummary {
    pub qualified_name: String,
    pub name: Option<String>,
    pub element_type: Option<String>,
    pub file_path: String,
}

/// Detail view for `GET /api/elements/{*qname}` — replaces the previous
/// hand-built `serde_json::json!{}` literal so this route uses the same
/// typed-struct + blanket-derive convention as every other JSON route
/// (`ElementSummary` above, `ChildSummary`/`ConnectionsResponse` in
/// `api_graph.rs`, the `validation.rs` DTOs).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementDetail {
    pub qualified_name: String,
    pub file_path: String,
    #[serde(rename = "type")]
    pub element_type: Option<String>,
    pub name: Option<String>,
    pub doc: String,
}

fn to_summary(e: &RawElement) -> ElementSummary {
    ElementSummary {
        qualified_name: e.qualified_name.clone(),
        name: e.frontmatter.name.clone(),
        element_type: e.frontmatter.element_type.as_ref().map(|t| format!("{:?}", t)),
        file_path: e.file_path.clone(),
    }
}

/// GET /api/elements[?type=PartDef]
pub async fn list_elements(
    State(state): State<SharedState>,
    Query(query): Query<ElementsQuery>,
) -> Json<Vec<ElementSummary>> {
    let store = state.read().await;
    let summaries: Vec<_> = store
        .elements
        .iter()
        .filter(|e| {
            if let Some(ref tf) = query.type_filter {
                e.frontmatter
                    .element_type
                    .as_ref()
                    .is_some_and(|t| format!("{:?}", t).eq_ignore_ascii_case(tf))
            } else {
                true
            }
        })
        .map(to_summary)
        .collect();
    Json(summaries)
}

/// GET /api/elements/:qname  (`:` in the path → use `*qname` wildcard)
pub async fn get_element(
    State(state): State<SharedState>,
    Path(qname): Path<String>,
) -> Result<Json<ElementDetail>, StatusCode> {
    let store = state.read().await;
    // Path segments use `/` — convert to `::`
    let qname_norm = qname.replace('/', "::");
    let element = store
        .elements
        .iter()
        .find(|e| e.qualified_name == qname_norm)
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(ElementDetail {
        qualified_name: element.qualified_name.clone(),
        file_path: element.file_path.clone(),
        element_type: element.frontmatter.element_type.as_ref().map(|t| format!("{:?}", t)),
        name: element.frontmatter.name.clone(),
        doc: element.doc.clone(),
    }))
}
