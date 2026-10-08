//! The feature model viewer (`REQ-TRS-FMED-001`, `-002`): `GET /features`, the
//! diagram, analysis and export endpoints under `/api/feature-model/`, over a
//! small product line with a dead feature, a core feature and a void variant.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

use syscribe_model::config::ValidateConfig;
use syscribe_model::walker::walk_model;
use syscribe_server::build_router;
use syscribe_server::state::new_state;

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn base(root: &Path) {
    write(root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(root, "F/_index.md", "---\ntype: Package\nname: F\n---\n");
    write(root, "F/Car.md", "---\ntype: FeatureDef\nid: FEAT-CAR\nname: Car\nmandatory: true\n---\n");
}

/// Car (mandatory) > Wheels (mandatory), Roof (optional), Spoiler (optional, excludes Wheels so dead).
fn healthy() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-fmpage-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&root).unwrap();
    base(&root);
    write(&root, "F/Car/Wheels.md", "---\ntype: FeatureDef\nid: FEAT-WHEELS\nname: Wheels\nmandatory: true\n---\n");
    write(&root, "F/Car/Roof.md", "---\ntype: FeatureDef\nid: FEAT-ROOF\nname: Roof\n---\n");
    write(&root, "F/Car/Spoiler.md", "---\ntype: FeatureDef\nid: FEAT-SPOILER\nname: Spoiler\nexcludes: [FEAT-WHEELS]\n---\n");
    root
}

fn void() -> PathBuf {
    let root = healthy();
    write(&root, "F/Car.md", "---\ntype: FeatureDef\nid: FEAT-CAR\nname: Car\nmandatory: true\nexcludes: [FEAT-WHEELS]\n---\n");
    root
}

async fn get(root: &Path, uri: &str) -> (StatusCode, String, String) {
    let elements = walk_model(root).unwrap();
    let config = ValidateConfig::with_model_root(root);
    let (shared, reload_tx) = new_state(elements, String::new(), config, root.to_path_buf());
    let app = build_router(shared, reload_tx);
    let resp = app.oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap()).await.unwrap();
    let status = resp.status();
    let mime = resp.headers().get("content-type").map(|v| v.to_str().unwrap().to_string()).unwrap_or_default();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    (status, mime, String::from_utf8_lossy(&bytes).into_owned())
}

async fn post_json(root: &Path, uri: &str, body: Value) -> Value {
    let elements = walk_model(root).unwrap();
    let config = ValidateConfig::with_model_root(root);
    let (shared, reload_tx) = new_state(elements, String::new(), config, root.to_path_buf());
    let app = build_router(shared, reload_tx);
    let resp = app
        .oneshot(Request::builder().method("POST").uri(uri).header("content-type", "application/json").body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

async fn json(root: &Path, uri: &str) -> Value {
    let (status, _, body) = get(root, uri).await;
    assert_eq!(status, StatusCode::OK, "{uri}: {body}");
    serde_json::from_str(&body).unwrap()
}

#[tokio::test]
async fn the_page_is_served_with_its_script_toolbar_and_live_badge() {
    let (status, _, html) = get(&healthy(), "/features").await;
    assert_eq!(status, StatusCode::OK);
    for id in ["fm-host", "fm-canvas", "fm-search", "fm-collapse", "fm-expand", "fm-fit", "fm-banner", "fm-summary", "fm-selected", "fm-live", "fm-empty", "fm-configure", "fm-config", "fm-edit", "fm-undo", "fm-redo", "fm-edit-panel", "fm-confirm", "fm-toast"] {
        assert!(html.contains(&format!("id=\"{id}\"")), "#{id} is in the page: {html}");
    }
    assert!(html.contains("/static/js/feature-model.js"), "{html}");
    assert!(html.contains("/static/js/live-reload.js"), "reload events come from the shared client");
    assert!(html.contains("format=svg") && html.contains("format=plantuml") && html.contains("format=mermaid"), "exports");
    assert!(html.contains("href=\"/features\""), "header link");
}

#[tokio::test]
async fn the_diagram_endpoint_returns_the_feature_tree_as_a_sprotty_graph() {
    let g = json(&healthy(), "/api/feature-model/diagram").await;
    assert_eq!(g["diagramKind"], "FeatureModel");
    let kids = g["children"].as_array().unwrap();
    let nodes: Vec<&Value> = kids.iter().filter(|c| c["type"] == "node").collect();
    assert_eq!(nodes.len(), 4);
    let wheels = nodes.iter().find(|n| n["name"] == "Wheels").unwrap();
    assert_eq!(wheels["feature"]["mandatory"], true);
    assert_eq!(wheels["ref"], "F::Car::Wheels");
    let edges: Vec<&Value> = kids.iter().filter(|c| c["type"] == "edge").collect();
    assert_eq!(edges.iter().filter(|e| e["kind"] == "child").count(), 3);
    assert_eq!(edges.iter().filter(|e| e["kind"] == "excludes").count(), 1);
    // A subtree.
    let sub = json(&healthy(), "/api/feature-model/diagram?root=F/Car/Roof").await;
    assert_eq!(sub["children"].as_array().unwrap().iter().filter(|c| c["type"] == "node").count(), 1);
}

#[tokio::test]
async fn the_analysis_endpoint_names_the_dead_feature_and_why() {
    let a = json(&healthy(), "/api/feature-model/analysis").await;
    assert_eq!(a["void"], false);
    assert_eq!(a["features"]["F::Car::Spoiler"]["state"], "dead", "{a}");
    let why = a["features"]["F::Car::Spoiler"]["reasons"].to_string();
    assert!(why.contains("excludes"), "{why}");
    assert_eq!(a["features"]["F::Car::Wheels"]["state"], "core");
    assert_eq!(a["features"]["F::Car::Roof"]["state"], "normal");
    assert_eq!(a["counts"]["dead"], 1);
}

#[tokio::test]
async fn a_void_model_reports_the_conflict_and_corrections() {
    let a = json(&void(), "/api/feature-model/analysis").await;
    assert_eq!(a["void"], true, "{a}");
    assert!(!a["conflicts"].as_array().unwrap().is_empty());
    assert!(!a["diagnoses"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn a_model_without_features_is_empty_not_an_error() {
    let root = std::env::temp_dir().join(format!("syscribe-fmpage-empty-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    let a = json(&root, "/api/feature-model/analysis").await;
    assert_eq!(a["hasFeatureModel"], false);
    let g = json(&root, "/api/feature-model/diagram").await;
    assert_eq!(g["children"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn the_export_endpoint_serves_svg_plantuml_and_mermaid_and_refuses_other_formats() {
    let root = healthy();
    let (s, mime, svg) = get(&root, "/api/feature-model/export?format=svg").await;
    assert_eq!(s, StatusCode::OK);
    assert!(mime.starts_with("image/svg+xml"), "{mime}");
    assert!(svg.contains("class=\"syscribe-diagram FeatureModel\""), "{svg}");
    let (_, mime, puml) = get(&root, "/api/feature-model/export?format=plantuml").await;
    assert!(mime.starts_with("text/plain") && puml.contains("<<mandatory>>"), "{puml}");
    let (_, _, mmd) = get(&root, "/api/feature-model/export?format=mermaid").await;
    assert!(mmd.starts_with("flowchart TD"), "{mmd}");
    let (s, _, _) = get(&root, "/api/feature-model/export?format=pdf").await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
}

/// Car > Engine (XOR: Petrol, Electric), Electric requires Charger.
fn configurable() -> PathBuf {
    let root = healthy();
    // Replace the default model's Spoiler/Roof with an engine group.
    std::fs::remove_file(root.join("F/Car/Spoiler.md")).unwrap();
    write(&root, "F/Car/Engine.md", "---\ntype: FeatureDef\nid: FEAT-ENGINE\nname: Engine\nmandatory: true\ngroupKind: alternative\n---\n");
    write(&root, "F/Car/Engine/Petrol.md", "---\ntype: FeatureDef\nid: FEAT-PETROL\nname: Petrol\n---\n");
    write(&root, "F/Car/Engine/Electric.md", "---\ntype: FeatureDef\nid: FEAT-ELECTRIC\nname: Electric\nrequires: [FEAT-CHARGER]\n---\n");
    write(&root, "F/Car/Charger.md", "---\ntype: FeatureDef\nid: FEAT-CHARGER\nname: Charger\n---\n");
    write(&root, "C/_index.md", "---\ntype: Package\nname: C\n---\n");
    write(&root, "C/CONF-FAST-001.md", "---\ntype: Configuration\nid: CONF-FAST-001\nname: Fast\nstatus: approved\nfeatureModel: F\nfeatures:\n  FEAT-ELECTRIC: true\n---\n");
    root
}

#[tokio::test]
async fn configure_propagates_a_choice_and_counts_the_products_left() {
    let root = configurable();
    let all = post_json(&root, "/api/feature-model/configure", serde_json::json!({ "selection": {} })).await;
    assert_eq!(all["satisfiable"], true, "{all}");
    assert_eq!(all["features"]["F::Car::Wheels"]["state"], "forcedOn");
    assert_eq!(all["features"]["F::Car::Roof"]["state"], "free");
    let electric = post_json(&root, "/api/feature-model/configure", serde_json::json!({ "selection": { "FEAT-ELECTRIC": true } })).await;
    assert_eq!(electric["features"]["F::Car::Engine::Electric"]["state"], "selected");
    assert_eq!(electric["features"]["F::Car::Charger"]["state"], "forcedOn");
    assert_eq!(electric["features"]["F::Car::Engine::Petrol"]["state"], "forcedOff");
    assert!(electric["products"]["count"].as_u64().unwrap() < all["products"]["count"].as_u64().unwrap());
    assert_eq!(electric["completion"]["F::Car::Charger"], true);
}

#[tokio::test]
async fn configure_explains_a_conflict_with_the_choices_and_constraints_at_fault() {
    let root = configurable();
    let j = post_json(&root, "/api/feature-model/configure", serde_json::json!({ "selection": { "F::Car::Roof": true, "FEAT-ELECTRIC": true, "F::Car::Charger": false } })).await;
    assert_eq!(j["satisfiable"], false, "{j}");
    let choices: Vec<&str> = j["conflict"]["choices"].as_array().unwrap().iter().map(|c| c["feature"].as_str().unwrap()).collect();
    assert!(choices.contains(&"F::Car::Charger") && choices.contains(&"F::Car::Engine::Electric") && !choices.contains(&"F::Car::Roof"), "{choices:?}");
    assert!(j["conflict"]["constraints"].to_string().contains("requires"), "{j}");
}

#[tokio::test]
async fn stored_configurations_are_listed_for_loading() {
    let list = json(&configurable(), "/api/feature-model/configurations").await;
    let c = &list.as_array().unwrap()[0];
    assert_eq!(c["id"], "CONF-FAST-001");
    assert_eq!(c["selection"]["F::Car::Engine::Electric"], true);
}

#[tokio::test]
async fn a_saved_completion_is_a_valid_configuration_the_analysis_accepts() {
    // The configurator's save: the server's completed product, written through the
    // guarded `POST /api/elements`, must be a Configuration the model calls valid.
    let root = configurable();
    let elements = walk_model(&root).unwrap();
    let config = ValidateConfig::with_model_root(&root);
    let (shared, reload_tx) = new_state(elements, String::new(), config, root.to_path_buf());
    let app = build_router(shared, reload_tx);

    let send = |method: &str, uri: &str, body: Value| {
        Request::builder().method(method).uri(uri).header("content-type", "application/json").body(Body::from(body.to_string())).unwrap()
    };
    let resp = app.clone().oneshot(send("POST", "/api/feature-model/configure", serde_json::json!({ "selection": { "FEAT-ELECTRIC": true } }))).await.unwrap();
    let configured: Value = serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let mut features = serde_json::Map::new();
    for (q, v) in configured["completion"].as_object().unwrap() {
        features.insert(q.clone(), v.clone());
    }
    let body = serde_json::json!({
        "qname": "C::Saved",
        "type": "Configuration",
        "fields": { "name": "Saved", "status": "draft", "featureModel": configured["featureModel"], "features": features },
    });
    let resp = app.clone().oneshot(send("POST", "/api/elements", body)).await.unwrap();
    let written: Value = serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(written["written"], true, "{written}");

    let resp = app.clone().oneshot(Request::builder().uri("/api/feature-model/analysis").body(Body::empty()).unwrap()).await.unwrap();
    let analysis: Value = serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let invalid = analysis["invalidConfigurations"].to_string();
    assert!(!invalid.contains("CONF-GEN-001"), "the saved product is valid: {analysis}");
    assert!(invalid.contains("CONF-FAST-001"), "the hand-written partial configuration, which leaves a mandatory feature out, is not: {analysis}");

    let resp = app.oneshot(Request::builder().uri("/api/feature-model/configurations").body(Body::empty()).unwrap()).await.unwrap();
    let list: Value = serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let saved = list.as_array().unwrap().iter().find(|c| c["qname"] == "C::Saved").expect("the new configuration is listed");
    assert_eq!(saved["selection"]["F::Car::Engine::Electric"], true);
    assert_eq!(saved["selection"]["F::Car::Charger"], true, "the product includes what Electric requires");
}
