use axum::{extract::State, Json};
use serde::Serialize;
use syscribe_model::validator::{self, Severity};
use crate::state::SharedState;

#[derive(Serialize)]
pub struct FindingDto {
    pub code: &'static str,
    pub severity: &'static str,
    pub file: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct CoverageEntry {
    pub req_id: String,
    pub test_case_ids: Vec<String>,
}

#[derive(Serialize)]
pub struct ValidationResponse {
    pub element_count: usize,
    pub error_count: usize,
    pub warning_count: usize,
    pub findings: Vec<FindingDto>,
    pub verified_by: Vec<CoverageEntry>,
    pub derived_children: Vec<CoverageEntry>,
}

/// GET /api/validation
/// Runs the full validation pass on the in-memory model and returns findings +
/// reverse indices (verifiedBy, derivedChildren).
pub async fn get_validation(State(state): State<SharedState>) -> Json<ValidationResponse> {
    let store = state.read().await;
    let result = validator::validate(&store.elements);

    let findings: Vec<FindingDto> = result
        .findings
        .iter()
        .map(|f| FindingDto {
            code: f.code,
            severity: match f.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
            },
            file: f.file.clone(),
            message: f.message.clone(),
        })
        .collect();

    let mut verified_by: Vec<CoverageEntry> = result
        .verified_by
        .into_iter()
        .map(|(req_id, tcs)| CoverageEntry { req_id, test_case_ids: tcs })
        .collect();
    verified_by.sort_by(|a, b| a.req_id.cmp(&b.req_id));

    let mut derived_children: Vec<CoverageEntry> = result
        .derived_children
        .into_iter()
        .map(|(req_id, children)| CoverageEntry { req_id, test_case_ids: children })
        .collect();
    derived_children.sort_by(|a, b| a.req_id.cmp(&b.req_id));

    Json(ValidationResponse {
        element_count: store.elements.len(),
        error_count: findings.iter().filter(|f| f.severity == "error").count(),
        warning_count: findings.iter().filter(|f| f.severity == "warning").count(),
        findings,
        verified_by,
        derived_children,
    })
}
