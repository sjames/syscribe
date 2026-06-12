#![deny(warnings)]

//! Library surface of the Syscribe model browser server.
//!
//! Exposes the route modules, shared state and the router constructor so that
//! integration tests can drive the same router `main` serves, in-process, via
//! `tower::ServiceExt::oneshot` (REQ-TRS-LINK-005).

use axum::{
    routing::{get, patch},
    Router,
};

pub mod routes;
pub mod state;
pub mod static_assets;

use routes::api_graph::{get_children, get_connections};
use routes::elements::{get_element, list_elements};
use routes::graph_cytoscape::get_graph;
use routes::ui::{canvas, diagram, element_detail, index, tree_items};
use routes::validation::get_validation;
use routes::write::{patch_layout, put_element};
use routes::ws::ws_handler;
use state::{ReloadTx, SharedState};
use tower_http::cors::CorsLayer;

/// Build the Axum router over an existing shared state and reload channel.
///
/// Extracted from `main` so integration tests can construct the same router
/// and drive it in-process via `tower::ServiceExt::oneshot` (REQ-TRS-LINK-005).
pub fn build_router(shared: SharedState, reload_tx: ReloadTx) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/canvas", get(canvas))
        .route("/ui/tree", get(tree_items))
        .route("/ui/detail/{*qname}", get(element_detail))
        .route("/ui/diagram/{*qname}", get(diagram))
        .route("/api/elements", get(list_elements))
        .route("/api/elements/{*qname}", get(get_element).put(put_element))
        .route("/api/children", get(get_children))
        .route("/api/connections", get(get_connections))
        .route("/api/diagrams/layout/{*qname}", patch(patch_layout))
        .route("/api/graph", get(get_graph))
        .route("/api/validation", get(get_validation))
        .route("/ws", get(ws_handler))
        .route("/static/{*path}", get(static_assets::static_handler))
        .layer(axum::Extension(reload_tx))
        .layer(CorsLayer::permissive())
        .with_state(shared)
}
