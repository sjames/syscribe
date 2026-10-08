//! `POST /api/feature-model/edit` (`REQ-TRS-FMED-004`): semantic edits through the
//! guarded-write engine, with a preview of the effect on the model's validity,
//! a confirmation for an edit that makes it worse, and an undo operation.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use axum::body::Body;
use axum::http::Request;
use axum::Router;
use http_body_util::BodyExt;
use serde_json::{json, Value};
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

/// Car (mandatory) > Engine (mandatory, alternative: Petrol, Electric), Charger; Electric requires Charger.
fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-fmedit-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&r).unwrap();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "F/_index.md", "---\ntype: Package\nname: F\n---\n");
    write(&r, "F/Car.md", "---\ntype: FeatureDef\nid: FEAT-CAR\nname: Car\nmandatory: true\n---\n");
    write(&r, "F/Car/Engine.md", "---\ntype: FeatureDef\nid: FEAT-ENGINE\nname: Engine\nmandatory: true\ngroupKind: alternative\n---\n");
    write(&r, "F/Car/Engine/Petrol.md", "---\ntype: FeatureDef\nid: FEAT-PETROL\nname: Petrol\n---\n");
    write(&r, "F/Car/Engine/Electric.md", "---\ntype: FeatureDef\nid: FEAT-ELECTRIC\nname: Electric\nrequires: [FEAT-CHARGER]\n---\n");
    write(&r, "F/Car/Charger.md", "---\ntype: FeatureDef\nid: FEAT-CHARGER\nname: Charger\n---\n");
    r
}

fn app(root: &Path) -> Router {
    let elements = walk_model(root).unwrap();
    let config = ValidateConfig::with_model_root(root);
    let (shared, reload_tx) = new_state(elements, String::new(), config, root.to_path_buf());
    build_router(shared, reload_tx)
}

async fn call(app: &Router, method: &str, uri: &str, body: Option<Value>) -> Value {
    let mut b = Request::builder().method(method).uri(uri);
    let body = match body {
        Some(v) => {
            b = b.header("content-type", "application/json");
            Body::from(v.to_string())
        }
        None => Body::empty(),
    };
    let resp = app.clone().oneshot(b.body(body).unwrap()).await.unwrap();
    serde_json::from_slice(&resp.into_body().collect().await.unwrap().to_bytes()).unwrap()
}

async fn edit(app: &Router, op: Value, preview: bool, accept_worse: bool) -> Value {
    call(app, "POST", "/api/feature-model/edit", Some(json!({ "edit": op, "preview": preview, "acceptWorse": accept_worse }))).await
}

#[tokio::test]
async fn a_preview_reports_the_effect_and_writes_nothing() {
    let r = model();
    let app = app(&r);
    let resp = edit(&app, json!({"op": "add", "parent": "F::Car", "name": "Sunroof"}), true, false).await;
    assert_eq!(resp["preview"], true);
    assert_eq!(resp["written"], false);
    assert_eq!(resp["delta"]["worsens"], false, "{resp}");
    assert_eq!(resp["delta"]["countsAfter"]["features"], 6, "{resp}");
    assert!(!r.join("F/Car/Sunroof.md").exists(), "a preview changes nothing on disk");
}

#[tokio::test]
async fn a_committed_edit_is_written_reloaded_and_comes_with_its_undo() {
    let r = model();
    let app = app(&r);
    let resp = edit(&app, json!({"op": "add", "parent": "F::Car", "name": "Sunroof"}), false, false).await;
    assert_eq!(resp["written"], true, "{resp}");
    assert_eq!(resp["feature"], "F::Car::Sunroof");
    assert!(r.join("F/Car/Sunroof.md").exists());
    // The live model reloaded: the diagram has the new feature.
    let g = call(&app, "GET", "/api/feature-model/diagram", None).await;
    let names: Vec<&str> = g["children"].as_array().unwrap().iter().filter(|c| c["type"] == "node").map(|c| c["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"Sunroof"), "{names:?}");
    // Undo is itself an edit, and returns the redo.
    let undone = edit(&app, resp["undo"].clone(), false, false).await;
    assert_eq!(undone["written"], true, "{undone}");
    assert!(!r.join("F/Car/Sunroof.md").exists());
    let redone = edit(&app, undone["undo"].clone(), false, false).await;
    assert_eq!(redone["written"], true, "{redone}");
    assert!(r.join("F/Car/Sunroof.md").exists(), "redo brings it back");
}

#[tokio::test]
async fn an_edit_that_makes_the_model_worse_needs_confirmation() {
    let r = model();
    let app = app(&r);
    // Charger excludes Electric: Electric (which requires Charger) becomes dead.
    let op = json!({"op": "addConstraint", "feature": "FEAT-CHARGER", "kind": "excludes", "target": "FEAT-ELECTRIC"});
    let held = edit(&app, op.clone(), false, false).await;
    assert_eq!(held["written"], false, "{held}");
    assert_eq!(held["needsConfirmation"], true);
    assert_eq!(held["delta"]["newDead"], json!(["F::Car::Engine::Electric"]));
    assert!(!std::fs::read_to_string(r.join("F/Car/Charger.md")).unwrap().contains("excludes"), "not written");
    let accepted = edit(&app, op, false, true).await;
    assert_eq!(accepted["written"], true, "{accepted}");
    assert!(std::fs::read_to_string(r.join("F/Car/Charger.md")).unwrap().contains("excludes"));
    // The live analysis now shows it dead.
    let a = call(&app, "GET", "/api/feature-model/analysis", None).await;
    assert_eq!(a["features"]["F::Car::Engine::Electric"]["state"], "dead");
    // Undoing it is not worse, so it needs no confirmation.
    let fixed = edit(&app, accepted["undo"].clone(), false, false).await;
    assert_eq!(fixed["written"], true, "{fixed}");
    assert_eq!(fixed["delta"]["resolvedDead"], json!(["F::Car::Engine::Electric"]));
}

#[tokio::test]
async fn a_model_made_void_is_reported_as_such() {
    let r = model();
    let app = app(&r);
    let resp = edit(&app, json!({"op": "addConstraint", "feature": "FEAT-CAR", "kind": "excludes", "target": "FEAT-ENGINE"}), true, false).await;
    assert_eq!(resp["delta"]["becameVoid"], true, "{resp}");
    assert_eq!(resp["delta"]["worsens"], true);
}

#[tokio::test]
async fn an_edit_the_engine_refuses_says_why_and_changes_nothing() {
    let r = model();
    let app = app(&r);
    let resp = edit(&app, json!({"op": "remove", "feature": "F::Car::Engine"}), false, false).await;
    assert_eq!(resp["written"], false);
    assert_eq!(resp["needsConfirmation"], false);
    assert!(resp["reason"].as_str().unwrap().contains("below it"), "{resp}");
    assert!(r.join("F/Car/Engine.md").exists());
    let resp = edit(&app, json!({"op": "rename", "feature": "F::Nope", "name": "X"}), false, false).await;
    assert!(resp["reason"].as_str().unwrap().contains("not a feature"), "{resp}");
}

#[tokio::test]
async fn rename_and_move_go_through_the_guarded_write_and_keep_the_model_consistent() {
    let r = model();
    let app = app(&r);
    let renamed = edit(&app, json!({"op": "rename", "feature": "F::Car::Charger", "name": "Plug"}), false, false).await;
    assert_eq!(renamed["written"], true, "{renamed}");
    assert_eq!(renamed["feature"], "F::Car::Plug");
    let a = call(&app, "GET", "/api/feature-model/analysis", None).await;
    assert!(a["features"].get("F::Car::Plug").is_some() && a["features"].get("F::Car::Charger").is_none(), "{a}");
    // Moving Plug into the Engine's alternative group would leave Electric, which requires it,
    // sharing a one-of-two group with the feature it needs: the editor says so before writing.
    let into_group = edit(&app, json!({"op": "move", "feature": "F::Car::Plug", "newParent": "F::Car::Engine"}), false, false).await;
    assert_eq!(into_group["written"], false, "{into_group}");
    assert_eq!(into_group["needsConfirmation"], true);
    assert_eq!(into_group["delta"]["newDead"], json!(["F::Car::Engine::Electric"]), "{into_group}");
    assert!(r.join("F/Car/Plug.md").exists(), "nothing was moved");
    // Taking Petrol out of the group leaves Electric as the group's only child, so it is forced on:
    // the editor reports the features that stopped being optional.
    let thinned = edit(&app, json!({"op": "move", "feature": "F::Car::Engine::Petrol", "newParent": "F::Car"}), false, false).await;
    assert_eq!(thinned["written"], false, "{thinned}");
    assert_eq!(thinned["delta"]["newFalseOptional"], json!(["F::Car::Engine::Electric", "F::Car::Plug"]), "{thinned}");
    // A move that costs nothing needs no confirmation: Plug as a root of its own.
    let root_move = edit(&app, json!({"op": "move", "feature": "F::Car::Plug"}), false, false).await;
    assert_eq!(root_move["written"], true, "{root_move}");
    assert_eq!(root_move["feature"], "F::Plug");
    assert_eq!(root_move["delta"]["worsens"], false, "{root_move}");
}

#[tokio::test]
async fn a_malformed_edit_is_a_client_error_not_a_panic() {
    let r = model();
    let app = app(&r);
    let resp = app
        .clone()
        .oneshot(Request::builder().method("POST").uri("/api/feature-model/edit").header("content-type", "application/json").body(Body::from("{\"edit\":{\"op\":\"explode\"}}")).unwrap())
        .await
        .unwrap();
    assert!(resp.status().is_client_error(), "{}", resp.status());
}

#[tokio::test]
async fn sheet_entries_and_parameters_are_edited_through_the_endpoint() {
    let r = std::env::temp_dir().join(format!("syscribe-fmedit-sheet-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&r);
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(
        &r,
        "Features.md",
        "---\ntype: FeatureModel\nname: Features\nfeatureTree:\n  - name: Car\n    id: FEAT-CAR\n    mandatory: true\n  - name: Car.Engine\n  - name: Car.Charger\n---\n",
    );
    let app = app(&r);
    // Rename a sheet entry: the diagram and analysis follow, the id is kept.
    let renamed = edit(&app, json!({"op": "rename", "feature": "Features::Car::Engine", "name": "Motor"}), false, false).await;
    assert_eq!(renamed["written"], true, "{renamed}");
    assert_eq!(renamed["feature"], "Features::Car::Motor");
    let a = call(&app, "GET", "/api/feature-model/analysis", None).await;
    assert!(a["features"].get("Features::Car::Motor").is_some(), "{a}");
    // A parameter, then its declaration shows on the diagram node.
    let p = edit(&app, json!({"op": "setParameter", "feature": "Features::Car::Motor", "parameter": {"name": "kw", "type": "ScalarValues::Real", "range": "1..=9"}}), false, false).await;
    assert_eq!(p["written"], true, "{p}");
    let g = call(&app, "GET", "/api/feature-model/diagram", None).await;
    let motor = g["children"].as_array().unwrap().iter().find(|c| c["ref"] == "Features::Car::Motor").unwrap();
    assert_eq!(motor["feature"]["parameters"][0]["name"], "kw", "{motor}");
    // Undo restores the sheet exactly.
    let undone = edit(&app, p["undo"].clone(), false, false).await;
    assert_eq!(undone["written"], true, "{undone}");
    let g = call(&app, "GET", "/api/feature-model/diagram", None).await;
    let motor = g["children"].as_array().unwrap().iter().find(|c| c["ref"] == "Features::Car::Motor").unwrap();
    assert!(motor["feature"].get("parameters").is_none());
}
