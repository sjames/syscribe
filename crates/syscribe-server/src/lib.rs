#![deny(warnings)]

//! Library surface of the Syscribe model browser server.
//!
//! Exposes the route modules, shared state and the router constructor so that
//! integration tests can drive the same router `main` serves, in-process, via
//! `tower::ServiceExt::oneshot` (REQ-TRS-LINK-005).

use axum::{
    routing::{get, patch, post, put},
    Router,
};

pub mod planning;
pub mod routes;
pub mod state;
pub mod static_assets;

use routes::api_graph::{get_children, get_connections};
use routes::diagram_model::get_diagram_model;
use routes::elements::{get_element, list_elements};
use routes::mutate::{
    add_connection, add_shape, create_element, delete_element, delete_layout, patch_layout, put_svg,
    remove_connection, update_element,
};
use routes::ui::{diagram, element_card, element_detail, features_page, heat_page, index, planning, planning_board, tree_items};
use routes::validation::get_validation;
use routes::ws::ws_handler;
use state::{ReloadTx, SharedState};

/// Build the Axum router over an existing shared state and reload channel.
///
/// Extracted from `main` so integration tests can construct the same router
/// and drive it in-process via `tower::ServiceExt::oneshot` (REQ-TRS-LINK-005).
pub fn build_router(shared: SharedState, reload_tx: ReloadTx) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/ui/tree", get(tree_items))
        .route("/ui/detail/{*qname}", get(element_detail))
        .route("/ui/element-card/{*qname}", get(element_card))
        .route("/planning", get(planning))
        .route("/heat", get(heat_page))
        .route("/features", get(features_page))
        .route("/api/feature-model/diagram", get(routes::feature_model::diagram))
        .route("/api/feature-model/analysis", get(routes::feature_model::analysis))
        .route("/api/feature-model/configure", post(routes::feature_model::configure))
        .route("/api/feature-model/edit", post(routes::feature_model::edit))
        .route("/api/feature-model/impact", get(routes::feature_model::impact))
        .route("/api/feature-model/configurations", get(routes::feature_model::configurations))
        .route("/api/feature-model/export", get(routes::feature_model::export))
        .route("/ui/planning/board", get(planning_board))
        .route("/ui/diagram/{*qname}", get(diagram))
        .route("/api/elements", get(list_elements).post(create_element))
        .route(
            "/api/elements/{*qname}",
            get(get_element).put(update_element).delete(delete_element),
        )
        .route("/api/children", get(get_children))
        .route(
            "/api/connections",
            get(get_connections).post(add_connection).delete(remove_connection),
        )
        .route("/api/diagrams/layout/{*qname}", patch(patch_layout).delete(delete_layout))
        .route("/api/diagrams/shapes/{*qname}", post(add_shape))
        .route("/api/diagrams/svg/{*qname}", put(put_svg))
        .route("/api/diagrams/model/{*qname}", get(get_diagram_model))
        .route("/api/validation", get(get_validation))
        .route("/api/req-graph", get(routes::req_graph::get_req_graph))
        .route("/api/req-graph/overview", get(routes::req_graph::get_overview))
        .route("/ws", get(ws_handler))
        .route("/static/{*path}", get(static_assets::static_handler))
        .layer(axum::Extension(reload_tx))
        .with_state(shared)
}
