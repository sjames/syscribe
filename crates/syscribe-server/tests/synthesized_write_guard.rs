//! Regression coverage (REQ-TRS-FM-005 corner-case review) for the web
//! `PUT`/`DELETE /api/elements/{*qname}` routes on an element synthesized
//! from a shared sheet file (here, a `FeatureModel` sheet's `featureTree:`
//! entry — the same gap affects FMEA/TARA entries too). Both routes resolve
//! a qname to a `file_path` and rewrite/remove that whole file; pointed at a
//! synthesized element that file is the *sheet*, shared with every sibling
//! feature — `update_element` would patch the sheet's own top-level
//! frontmatter instead of the buried entry, and `delete_element` would
//! delete every sibling feature along with the targeted one. Both must now
//! refuse. See `crates/syscribe/tests/mcp_synthesized_write_guard.rs` for the
//! MCP-side (and `move_element`) coverage of the same underlying guard.

use std::path::{Path, PathBuf};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

use syscribe_model::config::ValidateConfig;
use syscribe_model::walker::walk_model;
use syscribe_server::build_router;
use syscribe_server::state::new_state;

fn temp_model() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("syscribe-server-synth-guard-{}-{}", std::process::id(), nanos));
    let features = dir.join("Features");
    std::fs::create_dir_all(&features).unwrap();
    std::fs::write(
        features.join("_index.md"),
        r#"---
type: FeatureModel
name: Features
featureTree:
  - name: Platform
    id: FEAT-PLATFORM-001
    mandatory: true
    groupKind: alternative
  - name: Platform.CortexM
    id: FEAT-CORTEXM-001
    groupKind: optional
---
"#,
    )
    .unwrap();
    std::fs::write(
        features.join("SafeMode.md"),
        "---\ntype: FeatureDef\nid: FEAT-SAFEMODE-001\nname: SafeMode\ngroupKind: optional\n---\n",
    )
    .unwrap();
    dir
}

async fn build_app(model_root: &Path) -> axum::Router {
    let elements = walk_model(model_root).expect("walk model");
    let config = ValidateConfig::with_model_root(model_root);
    let (shared, reload_tx) = new_state(elements, String::new(), config, model_root.to_path_buf());
    build_router(shared, reload_tx)
}

async fn call(app: &axum::Router, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    let bytes = body.map(|b| serde_json::to_vec(&b).unwrap());
    let mut builder = Request::builder().method(method).uri(uri);
    if bytes.is_some() {
        builder = builder.header("content-type", "application/json");
    }
    let req_body = match bytes {
        Some(b) => Body::from(b),
        None => Body::empty(),
    };
    let resp = app.clone().oneshot(builder.body(req_body).unwrap()).await.expect("router response");
    let status = resp.status();
    let out_bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&out_bytes).unwrap_or(Value::Null);
    (status, json)
}

#[tokio::test]
async fn update_element_refuses_on_a_synthesized_feature() {
    let model_root = temp_model();
    let sheet = model_root.join("Features/_index.md");
    let before = std::fs::read(&sheet).unwrap();
    let app = build_app(&model_root).await;

    let (status, resp) = call(
        &app,
        "PUT",
        "/api/elements/Features/Platform/CortexM",
        Some(json!({"fields": {"mandatory": true}, "dryRun": false})),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["written"], json!(false), "must refuse, got {resp:#?}");
    let reason = resp.get("reason").and_then(|r| r.as_str()).unwrap_or_default();
    assert!(reason.contains("synthesized"), "refusal names the reason: {resp:#?}");

    let after = std::fs::read(&sheet).unwrap();
    assert_eq!(before, after, "sheet file must be left byte-for-byte unchanged");

    let _ = std::fs::remove_dir_all(&model_root);
}

#[tokio::test]
async fn delete_element_refuses_on_a_synthesized_feature() {
    let model_root = temp_model();
    let sheet = model_root.join("Features/_index.md");
    let app = build_app(&model_root).await;

    let (status, resp) = call(&app, "DELETE", "/api/elements/Features/Platform/CortexM?dryRun=false", None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["written"], json!(false), "must refuse, got {resp:#?}");
    let reason = resp.get("reason").and_then(|r| r.as_str()).unwrap_or_default();
    assert!(reason.contains("synthesized"), "refusal names the reason: {resp:#?}");
    assert!(sheet.exists(), "sheet file (every sibling feature) must survive");

    let _ = std::fs::remove_dir_all(&model_root);
}

/// Positive control: a genuine per-file `FeatureDef` in the same model is
/// unaffected — proves the guard targets synthesis, not `FeatureDef` as a type.
#[tokio::test]
async fn update_element_still_works_on_a_genuine_per_file_feature() {
    let model_root = temp_model();
    let app = build_app(&model_root).await;

    let (status, resp) = call(
        &app,
        "PUT",
        "/api/elements/Features/SafeMode",
        Some(json!({"fields": {"mandatory": true}, "dryRun": false})),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["written"], json!(true), "expected commit, got {resp:#?}");
    let content = std::fs::read_to_string(model_root.join("Features/SafeMode.md")).unwrap();
    assert!(content.contains("mandatory: true"), "field actually updated:\n{content}");

    let _ = std::fs::remove_dir_all(&model_root);
}
