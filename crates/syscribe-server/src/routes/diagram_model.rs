//! `GET /api/diagrams/model/{*qname}` — the sprotty diagram-model endpoint
//! (`ADR-SYS-DE-001`, `REQ-TRS-DE-004`; nested shape per `REQ-TRS-VIS-006`,
//! `ADR-SYS-VIS-001`).
//!
//! The handler is deliberately thin: it resolves the element, builds the
//! Diagram IR through `syscribe_model::vis::build_graph` (the one manifest
//! parser, `REQ-TRS-VIS-001`) and hands the IR to
//! `syscribe_model::vis::sprotty::to_sgraph`, whose module doc is the JSON
//! contract — a **nested** `SGraph` with `port`/`label`/`compartment`
//! children, `layoutOptions` (ELK option ids) and the `pinned` set. Nothing
//! here reads `shapes:`/`edges:`/`layout:` frontmatter.
//!
//! `404` when `qname` is unknown, is not a `Diagram`, or is a hand-authored
//! `Mermaid`/`PlantUML` diagram (no IR; the UI never requests those — it
//! fetches the `/ui/diagram` HTML fragment instead). A `Diagram` with a
//! `subject:` and no `shapes:` returns `200` with the empty derived graph
//! (the client needs a valid, mountable root either way).
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

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use syscribe_model::vis::sprotty::{to_sgraph, SGraph};

use crate::state::SharedState;

/// `GET /api/diagrams/model/{*qname}` — see the module doc comment.
pub async fn get_diagram_model(
    State(state): State<SharedState>,
    Path(qname): Path<String>,
) -> Result<Json<SGraph>, StatusCode> {
    let store = state.read().await;
    let qname_norm = qname.replace('/', "::");
    let element = store
        .resolver
        .get(&store.elements, &qname_norm)
        .ok_or(StatusCode::NOT_FOUND)?;
    // `build_graph` is `None` for a non-`Diagram` and for the IR-less
    // `Mermaid`/`PlantUML` kinds alike. Manifest issues (`E405`/`W416`) are
    // the validator's to report; the client draws whatever survived them.
    let (graph, _issues) = syscribe_model::vis::build_graph(element, &store.elements, &store.resolver)
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(to_sgraph(&graph)))
}
