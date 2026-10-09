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
//! A fault tree, attack tree, safety goal, argument or threat scenario (GH #223)
//! has no `Diagram` element, but the model itself says what to draw: the
//! endpoint then returns its derived safety diagram
//! (`syscribe_model::vis::build_subject_graph`), the element's primary view or,
//! with `?kind=FaultTree|AttackTree|SafetyCase`, the named one among
//! `vis::safety_kinds_of`.
//!
//! `404` when `qname` is unknown, is neither a `Diagram` nor such an element, or
//! is a hand-authored `Mermaid`/`PlantUML` diagram (no IR; the UI never requests those — it
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
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};

use serde::Deserialize;
use syscribe_model::vis::sprotty::{to_sgraph, SGraph};
use syscribe_model::vis::DiagramKind;

use crate::state::SharedState;

#[derive(Deserialize)]
pub struct ModelQuery {
    /// For a non-`Diagram` safety element: which of its views (`FaultTree`,
    /// `AttackTree`, `SafetyCase`).
    pub kind: Option<String>,
}

/// `GET /api/diagrams/model/{*qname}` — see the module doc comment.
pub async fn get_diagram_model(
    State(state): State<SharedState>,
    Path(qname): Path<String>,
    Query(q): Query<ModelQuery>,
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
    if let Some((graph, _issues)) = syscribe_model::vis::build_graph(element, &store.elements, &store.resolver) {
        return Ok(Json(to_sgraph(&graph)));
    }
    // Not a `Diagram`: a safety element is drawn from the model (GH #223).
    let kinds = syscribe_model::vis::safety_kinds_of(element, &store.elements, &store.resolver);
    let kind = match q.kind.as_deref() {
        Some(k) => DiagramKind::parse(Some(k)).filter(|k| kinds.contains(k)),
        None => kinds.first().copied(),
    }
    .ok_or(StatusCode::NOT_FOUND)?;
    let (graph, _issues) = syscribe_model::vis::build_subject_graph(element, kind, &store.elements, &store.resolver);
    Ok(Json(to_sgraph(&graph)))
}
