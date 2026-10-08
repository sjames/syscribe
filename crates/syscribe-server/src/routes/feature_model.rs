//! The feature model viewer's data (`REQ-TRS-FMED-001`, `-002`, `ADR-SYS-FMED-001`).
//!
//! Everything here is a thin read over `syscribe_model`: the diagram is the
//! Diagram IR's `FeatureModel` kind (`vis::derive::feature`) serialised as the
//! sprotty model the diagram editor already speaks, the analysis is the SAT
//! engine's structured report (`feature_model::analysis_json`), and the export
//! is the IR's PlantUML, Mermaid and SVG writers. The browser never solves.

use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use syscribe_model::vis::{derive::feature::feature_diagram, sprotty::{to_sgraph, SGraph}};

use crate::state::SharedState;

#[derive(Deserialize, Default)]
pub struct RootQuery {
    /// A `FeatureDef` qualified name: draw only its subtree.
    pub root: Option<String>,
}

#[derive(Deserialize)]
pub struct ExportQuery {
    pub root: Option<String>,
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_format() -> String {
    "svg".to_string()
}

fn root_of(q: &RootQuery) -> Option<String> {
    q.root.as_ref().map(|r| r.replace('/', "::")).filter(|r| !r.is_empty())
}

/// `GET /api/feature-model/diagram[?root=<qname>]` — the feature diagram as a
/// sprotty graph (empty when the model has no features).
pub async fn diagram(State(state): State<SharedState>, Query(q): Query<RootQuery>) -> Json<SGraph> {
    let store = state.read().await;
    let graph = feature_diagram(&store.elements, root_of(&q).as_deref());
    Json(to_sgraph(&graph))
}

/// `GET /api/feature-model/analysis` — void/dead/core/false-optional with reasons.
pub async fn analysis(State(state): State<SharedState>) -> Json<Value> {
    let store = state.read().await;
    Json(syscribe_model::feature_model::analysis_json(&store.elements))
}

#[derive(Deserialize, Default)]
pub struct ConfigureRequest {
    /// Feature (qualified name or `FEAT-*` id) → the user's choice.
    #[serde(default)]
    pub selection: std::collections::BTreeMap<String, bool>,
}

/// `POST /api/feature-model/configure` — propagate a partial selection
/// (`REQ-TRS-FMED-003`): each feature's state, whether the selection can still be
/// completed and, if not, which choices clash and why, the number of products left
/// and one complete product consistent with the choices. Read-only.
pub async fn configure(State(state): State<SharedState>, Json(req): Json<ConfigureRequest>) -> Json<Value> {
    let store = state.read().await;
    Json(syscribe_model::feature_model::configure_selection(&store.elements, &req.selection))
}

/// `GET /api/feature-model/configurations` — the stored `Configuration`s with
/// their selections, for the configurator's load list.
pub async fn configurations(State(state): State<SharedState>) -> Json<Value> {
    let store = state.read().await;
    Json(syscribe_model::feature_model::configurations_json(&store.elements))
}

/// `GET /api/feature-model/export?format=svg|plantuml|mermaid[&root=<qname>]`.
pub async fn export(State(state): State<SharedState>, Query(q): Query<ExportQuery>) -> Response {
    let store = state.read().await;
    let root = root_of(&RootQuery { root: q.root.clone() });
    let graph = feature_diagram(&store.elements, root.as_deref());
    let links = |qname: &str| -> Option<String> {
        let target = store.resolver.resolve_ref(&store.elements, qname)?;
        store.config.hosted_url_for(&target.file_path, &target.qualified_name, target.frontmatter.id.as_deref().unwrap_or(""))
    };
    let (body, mime, ext) = match q.format.as_str() {
        "svg" => match syscribe_model::vis::svg::render_svg(&graph, &links) {
            Ok(s) => (s, "image/svg+xml", "svg"),
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("could not lay out the feature model: {e}")).into_response(),
        },
        "mermaid" => match syscribe_model::vis::mermaid::render_mermaid(&graph, &links) {
            Some(s) => (s, "text/plain; charset=utf-8", "mmd"),
            None => return StatusCode::NOT_FOUND.into_response(),
        },
        "plantuml" => (syscribe_model::plantuml::render_feature_model_plantuml(&graph, "FeatureModel"), "text/plain; charset=utf-8", "puml"),
        other => return (StatusCode::BAD_REQUEST, json!({ "error": format!("unknown format '{other}': use svg, plantuml or mermaid") }).to_string()).into_response(),
    };
    (
        [
            (header::CONTENT_TYPE, mime.to_string()),
            (header::CONTENT_DISPOSITION, format!("inline; filename=\"feature-model.{ext}\"")),
        ],
        body,
    )
        .into_response()
}
