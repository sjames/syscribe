//! Integration tests for the diagram-driven structural-edit endpoints
//! (`ADR-SYS-DE-001`, `REQ-TRS-DE-002`/`003`/`005`) in `routes::mutate`.
//!
//! Driven in-process against the real router via `tower::ServiceExt::oneshot`,
//! built exactly as `main` builds it (`build_router` + `new_state`), following
//! the same pattern as `tests/source_link.rs`. Each test copies the shared
//! `tests/fixtures/mutate` model into a fresh temp directory so commits are
//! real filesystem writes without ever touching the checked-in fixture or
//! racing other tests.

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

fn fixtures_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mutate")
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// Copy the shared fixture model into a fresh temp directory so this test can
/// commit real writes without touching the checked-in fixture.
fn temp_model() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "syscribe-server-mutate-test-{}-{}",
        std::process::id(),
        nanos
    ));
    copy_dir_all(&fixtures_root(), &dir).expect("copy fixture model");
    dir
}

async fn build_app(model_root: &Path) -> axum::Router {
    let elements = walk_model(model_root).expect("walk fixture model");
    let config = ValidateConfig::with_model_root(model_root);
    let (shared, reload_tx) = new_state(elements, String::new(), config, model_root.to_path_buf());
    build_router(shared, reload_tx)
}

/// GET `uri` against `app` and return the raw HTML/text body (for `/ui/*`
/// fragment routes, which don't return JSON).
async fn get_html(app: &axum::Router, uri: &str) -> String {
    let resp = app
        .clone()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .expect("router response");
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8(bytes.to_vec()).expect("utf8 body")
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
    let resp = app
        .clone()
        .oneshot(builder.body(req_body).unwrap())
        .await
        .expect("router response");
    let status = resp.status();
    let out_bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&out_bytes).unwrap_or(Value::Null);
    (status, json)
}

/// `REQ-TRS-DE-002`/`003` — creating an element with diagram context commits
/// the new file and patches the diagram's `shapes:`/`layout:` in one call.
#[tokio::test]
async fn create_with_diagram_sync_commits_and_updates_diagram() {
    let model_root = temp_model();
    let app = build_app(&model_root).await;

    let (status, resp) = call(
        &app,
        "POST",
        "/api/elements",
        Some(json!({
            "qname": "Basics/NewPart",
            "type": "PartDef",
            "fields": {"name": "NewPart"},
            "diagram": {
                "qname": "Diagrams::TestDiagram",
                "shapeId": "s-newpart",
                "x": 50.0,
                "y": 60.0,
                "kind": "PartDef"
            },
            "dryRun": false
        })),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["written"], json!(true), "expected commit, got {resp:#?}");

    let new_file = model_root.join("Basics/NewPart.md");
    assert!(new_file.exists(), "new element file should exist on disk");

    let diagram_content =
        std::fs::read_to_string(model_root.join("Diagrams/TestDiagram.md")).unwrap();
    assert!(
        diagram_content.contains("s-newpart"),
        "diagram should gain the new shape id:\n{diagram_content}"
    );
    assert!(
        diagram_content.contains("Basics::NewPart"),
        "diagram shape should ref the new qname:\n{diagram_content}"
    );

    let _ = std::fs::remove_dir_all(&model_root);
}

/// `REQ-TRS-DE-005` — a create that would introduce a dangling reference
/// (`supertype` pointing at a nonexistent element) is refused: disk untouched,
/// `newErrors` populated so the client can show what broke.
#[tokio::test]
async fn create_with_dangling_reference_is_refused() {
    let model_root = temp_model();
    let app = build_app(&model_root).await;

    let (status, resp) = call(
        &app,
        "POST",
        "/api/elements",
        Some(json!({
            "qname": "Basics/BadPart",
            "type": "PartDef",
            "fields": {"name": "BadPart", "supertype": "Nonexistent::Ghost"},
            "dryRun": false
        })),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["written"], json!(false), "expected refusal, got {resp:#?}");
    assert!(
        !resp["newErrors"].as_array().unwrap().is_empty(),
        "expected a newErrors entry for the dangling supertype, got {resp:#?}"
    );
    assert!(
        !model_root.join("Basics/BadPart.md").exists(),
        "a refused write must never touch disk"
    );

    let _ = std::fs::remove_dir_all(&model_root);
}

/// `REQ-TRS-DE-002` — add and remove a `connections:` entry on the owning
/// element, round-tripping through `/api/connections`.
#[tokio::test]
async fn connection_add_then_remove_round_trips() {
    let model_root = temp_model();
    let app = build_app(&model_root).await;

    let (status, resp) = call(
        &app,
        "POST",
        "/api/connections",
        Some(json!({
            "qname": "Basics::Widget",
            "from": "a.out",
            "to": "b.in",
            "dryRun": false
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["written"], json!(true), "expected add commit, got {resp:#?}");

    let content = std::fs::read_to_string(model_root.join("Basics/Widget.md")).unwrap();
    assert!(
        content.contains("a.out") && content.contains("b.in"),
        "connection should be recorded:\n{content}"
    );

    let (status, resp) = call(
        &app,
        "DELETE",
        "/api/connections",
        Some(json!({
            "qname": "Basics::Widget",
            "from": "a.out",
            "to": "b.in",
            "dryRun": false
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["written"], json!(true), "expected remove commit, got {resp:#?}");

    let content = std::fs::read_to_string(model_root.join("Basics/Widget.md")).unwrap();
    assert!(
        !content.contains("a.out"),
        "connection should be removed:\n{content}"
    );

    let _ = std::fs::remove_dir_all(&model_root);
}

/// `REQ-TRS-DE-002`/`005` — deleting an element blocked by an inbound
/// reference is refused via this module's one response convention: always
/// `StatusCode::OK`, `written:false`, and a populated `blockedBy` list (no
/// `409` — that shape was retired along with `DeleteBlockedResponse`).
#[tokio::test]
async fn delete_blocked_by_referrer_requires_force() {
    let model_root = temp_model();
    // Give Widget a subtype so it has an inbound referrer.
    std::fs::create_dir_all(model_root.join("Basics")).unwrap();
    std::fs::write(
        model_root.join("Basics/Gadget.md"),
        "---\ntype: PartDef\nname: Gadget\nsupertype: Basics::Widget\n---\n\nA gadget.\n",
    )
    .unwrap();

    let app = build_app(&model_root).await;

    let (status, resp) = call(&app, "DELETE", "/api/elements/Basics/Widget", None).await;
    assert_eq!(status, StatusCode::OK, "blocked delete stays a 200, got {resp:#?}");
    assert_eq!(resp["written"], json!(false));
    assert!(
        !resp["blockedBy"].as_array().unwrap().is_empty(),
        "expected Gadget listed as a blocker, got {resp:#?}"
    );
    assert!(
        model_root.join("Basics/Widget.md").exists(),
        "blocked delete must not touch disk"
    );

    let (status, resp) = call(&app, "DELETE", "/api/elements/Basics/Widget?force=true", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["written"], json!(true), "force delete should commit, got {resp:#?}");
    assert!(!model_root.join("Basics/Widget.md").exists());

    let _ = std::fs::remove_dir_all(&model_root);
}

/// `delete_element` on an unresolved qname: same convention, `written:false`
/// + `reason`, no `blockedBy` entries (nothing to be blocked by an element
/// that doesn't exist) — replaces the retired `404`.
#[tokio::test]
async fn delete_element_unresolved_qname_is_written_false_not_404() {
    let model_root = temp_model();
    let app = build_app(&model_root).await;

    let (status, resp) = call(&app, "DELETE", "/api/elements/Nonexistent/Ghost", None).await;
    assert_eq!(status, StatusCode::OK, "unresolved delete stays a 200, got {resp:#?}");
    assert_eq!(resp["written"], json!(false));
    assert!(resp["blockedBy"].as_array().unwrap().is_empty());
    assert!(resp["reason"].as_str().is_some());

    let _ = std::fs::remove_dir_all(&model_root);
}

/// `update_element` happy path: `PUT /api/elements/{*qname}` merges a field
/// change into frontmatter and returns the same `WriteResponse` shape every
/// other guarded-write handler returns (not the retired `{ok:true}`).
#[tokio::test]
async fn update_element_commits_field_change_and_returns_write_response() {
    let model_root = temp_model();
    let app = build_app(&model_root).await;

    let (status, resp) = call(
        &app,
        "PUT",
        "/api/elements/Basics/Widget",
        Some(json!({
            "fields": {"name": "RenamedWidget", "status": "approved"},
            "doc": "Updated documentation.",
            "dryRun": false
        })),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["written"], json!(true), "expected commit, got {resp:#?}");
    assert!(resp.get("newErrors").is_some(), "expected WriteResponse shape, got {resp:#?}");

    let content = std::fs::read_to_string(model_root.join("Basics/Widget.md")).unwrap();
    assert!(content.contains("name: RenamedWidget"), "name should be updated:\n{content}");
    assert!(content.contains("status: approved"), "status should be added:\n{content}");
    assert!(content.contains("Updated documentation."), "doc should be replaced:\n{content}");

    let _ = std::fs::remove_dir_all(&model_root);
}

/// `update_element` gate path: setting a reference field to an unresolved
/// qname is refused (`written:false`, populated `newErrors`) and disk is left
/// untouched — mirrors `create_with_dangling_reference_is_refused`.
#[tokio::test]
async fn update_element_refuses_when_new_reference_unresolved() {
    let model_root = temp_model();
    let app = build_app(&model_root).await;

    let before = std::fs::read_to_string(model_root.join("Basics/Widget.md")).unwrap();

    let (status, resp) = call(
        &app,
        "PUT",
        "/api/elements/Basics/Widget",
        Some(json!({
            "fields": {"supertype": "Nonexistent::Ghost"},
            "dryRun": false
        })),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["written"], json!(false), "expected refusal, got {resp:#?}");
    assert!(
        !resp["newErrors"].as_array().unwrap().is_empty(),
        "expected a newErrors entry for the dangling supertype, got {resp:#?}"
    );

    let after = std::fs::read_to_string(model_root.join("Basics/Widget.md")).unwrap();
    assert_eq!(before, after, "a refused write must never touch disk");

    let _ = std::fs::remove_dir_all(&model_root);
}

/// `patch_layout` happy path, rebuilt on `guarded_write` — asserts the
/// `layout:` block lands on disk and the response is the new `WriteResponse`
/// shape, not the retired `{ok:true}`.
#[tokio::test]
async fn patch_layout_commits_via_guarded_write() {
    let model_root = temp_model();
    let app = build_app(&model_root).await;

    let (status, resp) = call(
        &app,
        "PATCH",
        "/api/diagrams/layout/Diagrams/TestDiagram",
        Some(json!({
            "s-widget": {"x": 123.0, "y": 456.0}
        })),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["written"], json!(true), "expected commit, got {resp:#?}");
    assert!(resp.get("newErrors").is_some(), "expected WriteResponse shape, got {resp:#?}");

    let content = std::fs::read_to_string(model_root.join("Diagrams/TestDiagram.md")).unwrap();
    assert!(content.contains("x: 123"), "layout x should be updated:\n{content}");
    assert!(content.contains("y: 456"), "layout y should be updated:\n{content}");

    let _ = std::fs::remove_dir_all(&model_root);
}

/// `patch_layout` on an unknown qname: moved off the retired `404` onto this
/// module's one always-`OK`/`written:false` convention (see the reasoning in
/// `routes::mutate::patch_layout`'s doc comment) rather than silently
/// no-op'ing — the refusal is now reported in the body, not the status code.
#[tokio::test]
async fn patch_layout_unknown_qname_is_written_false_not_404() {
    let model_root = temp_model();
    let app = build_app(&model_root).await;

    let (status, resp) = call(
        &app,
        "PATCH",
        "/api/diagrams/layout/Nonexistent/Ghost",
        Some(json!({ "s-x": {"x": 1.0, "y": 1.0} })),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "unresolved-qname patch stays a 200, got {resp:#?}");
    assert_eq!(resp["written"], json!(false));
    assert!(resp["reason"].as_str().is_some());

    let _ = std::fs::remove_dir_all(&model_root);
}

/// Phase 8 — the edit-mode markup gained an `#edit-extra` YAML textarea,
/// pre-filled with the element's on-disk frontmatter minus `name` (the name
/// stays in its own `#edit-name` field). Guards against silent template
/// breakage now that `saveElement()` depends on this element existing.
#[tokio::test]
async fn element_detail_edit_mode_has_extra_yaml_textarea() {
    let model_root = temp_model();
    let app = build_app(&model_root).await;

    let html = get_html(&app, "/ui/detail/Basics/Widget").await;

    assert!(
        html.contains(r#"id="edit-extra""#),
        "expected the extra-fields textarea, got:\n{html}"
    );
    assert!(
        html.contains("type: PartDef"),
        "extra-fields textarea should include non-name frontmatter, got:\n{html}"
    );
    // `name` is edited via its own field, not duplicated into the YAML blob.
    let extra_start = html.find(r#"id="edit-extra""#).unwrap();
    let extra_section = &html[extra_start..];
    let extra_close = extra_section.find("</textarea>").unwrap();
    assert!(
        !extra_section[..extra_close].contains("name: Widget"),
        "extra-fields textarea should not duplicate name, got:\n{html}"
    );

    let _ = std::fs::remove_dir_all(&model_root);
}

/// `ModelStore::commit` (every mutating handler now goes through it, see
/// `state.rs`) pushes a `{"event":"reload"}` message onto the store's
/// `reload_tx` broadcast channel on every successful commit, in addition to
/// reloading in-memory state — this is new coverage, nothing exercised the
/// broadcast side of a commit before this task's refactor.
#[tokio::test]
async fn successful_commit_broadcasts_reload_event() {
    let model_root = temp_model();
    let elements = walk_model(&model_root).expect("walk fixture model");
    let config = ValidateConfig::with_model_root(&model_root);
    let (shared, reload_tx) = new_state(elements, String::new(), config, model_root.clone());
    // Subscribe before issuing the request so the broadcast can't be missed.
    let mut rx = reload_tx.subscribe();
    let app = build_router(shared, reload_tx);

    let (status, resp) = call(
        &app,
        "PUT",
        "/api/elements/Basics/Widget",
        Some(json!({
            "fields": {"name": "RenamedWidget"},
            "dryRun": false
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["written"], json!(true), "expected commit, got {resp:#?}");

    let msg = tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv())
        .await
        .expect("reload broadcast should arrive promptly")
        .expect("reload channel should not be closed");
    assert_eq!(msg, r#"{"event":"reload"}"#);

    let _ = std::fs::remove_dir_all(&model_root);
}

/// Phase 1 (dead-code deletion) regression guard: the retired Cytoscape
/// `/canvas` explorer and its `/api/graph` endpoint must stay gone — a
/// cheap check against accidental re-registration in `build_router`.
#[tokio::test]
async fn retired_canvas_routes_are_404() {
    let model_root = temp_model();
    let app = build_app(&model_root).await;

    let (status, _) = call(&app, "GET", "/canvas", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "/canvas must not be registered");

    let (status, _) = call(&app, "GET", "/api/graph", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "/api/graph must not be registered");

    let _ = std::fs::remove_dir_all(&model_root);
}
