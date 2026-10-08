//! Integration tests for creating a new `Diagram` from the browser's dialog
//! (`REQ-TRS-VIS-023`): the dialog posts `{qname, type: "Diagram", fields}` to
//! `POST /api/elements`, and these tests pin what the server does with the two
//! shapes it sends:
//!
//! - a **derived** diagram (`diagramKind` + `subject`, no `shapes`) is written,
//!   served as a generated graph and flagged `derived`;
//! - a **blank** diagram (`shapes: {}`) is written, served as an empty
//!   manifest graph, and then accepts an Add through the existing diagram-sync
//!   path;
//! - a name already in use is refused and writes nothing;
//! - validation warnings the new element raises (`W418` subject type, `W401`
//!   unresolved subject) come back in `newWarnings` for the dialog to show.
//!
//! Same in-process pattern as `tests/layout_routes.rs`.

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

/// A unique temp copy of the fixture model (see `tests/mutate.rs` for why the
/// per-process sequence number is needed).
fn temp_model() -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = loop {
        let dir = std::env::temp_dir().join(format!(
            "syscribe-server-newdiagram-test-{}-{}-{}",
            std::process::id(),
            nanos,
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        match std::fs::create_dir(&dir) {
            Ok(()) => break dir,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => panic!("create temp model dir: {e}"),
        }
    };
    copy_dir_all(&fixtures_root(), &dir).expect("copy fixture model");
    dir
}


async fn build_app(model_root: &Path) -> axum::Router {
    let elements = walk_model(model_root).expect("walk fixture model");
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

fn warning_codes(resp: &Value) -> Vec<String> {
    resp["newWarnings"]
        .as_array()
        .map(|a| a.iter().filter_map(|f| f["code"].as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

#[tokio::test]
async fn a_derived_diagram_is_written_and_served_as_a_generated_graph() {
    let root = temp_model();
    let app = build_app(&root).await;
    let (_, resp) = call(
        &app,
        "POST",
        "/api/elements",
        Some(json!({
            "qname": "Diagrams::WidgetIBD",
            "type": "Diagram",
            "fields": { "diagramKind": "IBD", "subject": "Basics::Widget" }
        })),
    )
    .await;
    assert_eq!(resp["written"], json!(true), "{resp}");

    let text = std::fs::read_to_string(root.join("Diagrams/WidgetIBD.md")).expect("diagram file written");
    assert!(text.contains("type: Diagram"), "{text}");
    assert!(text.contains("diagramKind: IBD"), "{text}");
    assert!(text.contains("subject: Basics::Widget"), "{text}");
    assert!(!text.contains("shapes"), "a derived diagram lists no shapes:\n{text}");

    let (status, graph) = call(&app, "GET", "/api/diagrams/model/Diagrams/WidgetIBD", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(graph["derived"], json!(true));
    assert_eq!(graph["subject"], "Basics::Widget");
    let children = graph["children"].as_array().unwrap();
    assert_eq!(children.len(), 1, "the IBD boundary of the subject: {graph}");
    assert_eq!(children[0]["id"], "s-basics-widget");
    assert_eq!(children[0]["kind"], "boundary");
}

#[tokio::test]
async fn a_blank_diagram_is_an_empty_manifest_that_accepts_an_add() {
    let root = temp_model();
    let app = build_app(&root).await;
    let (_, resp) = call(
        &app,
        "POST",
        "/api/elements",
        Some(json!({
            "qname": "Diagrams::Scratch",
            "type": "Diagram",
            "fields": { "diagramKind": "BDD", "shapes": {} }
        })),
    )
    .await;
    assert_eq!(resp["written"], json!(true), "{resp}");
    let text = std::fs::read_to_string(root.join("Diagrams/Scratch.md")).unwrap();
    assert!(text.contains("shapes: {}"), "an explicit empty manifest selects the manifest source:\n{text}");

    let (status, graph) = call(&app, "GET", "/api/diagrams/model/Diagrams/Scratch", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(graph["derived"], json!(false));
    assert_eq!(graph["children"], json!([]));

    // Add: a new element plus its shape and pin in the blank diagram, in one write.
    let (_, add) = call(
        &app,
        "POST",
        "/api/elements",
        Some(json!({
            "qname": "Basics::Gizmo",
            "type": "PartDef",
            "diagram": { "qname": "Diagrams::Scratch", "shapeId": "s-gizmo", "x": 10, "y": 20, "kind": "PartDef" }
        })),
    )
    .await;
    assert_eq!(add["written"], json!(true), "{add}");
    let (_, graph) = call(&app, "GET", "/api/diagrams/model/Diagrams/Scratch", None).await;
    let children = graph["children"].as_array().unwrap();
    assert_eq!(children.len(), 1, "{graph}");
    assert_eq!(children[0]["id"], "s-gizmo");
    assert_eq!(graph["pinned"], json!(["s-gizmo"]));
}

#[tokio::test]
async fn a_name_already_in_use_is_refused_and_nothing_is_written() {
    let root = temp_model();
    let before = std::fs::read_to_string(root.join("Diagrams/TestDiagram.md")).unwrap();
    let app = build_app(&root).await;
    let (_, resp) = call(
        &app,
        "POST",
        "/api/elements",
        Some(json!({
            "qname": "Diagrams::TestDiagram",
            "type": "Diagram",
            "fields": { "diagramKind": "IBD", "subject": "Basics::Widget" }
        })),
    )
    .await;
    assert_eq!(resp["written"], json!(false), "{resp}");
    assert!(resp["reason"].as_str().is_some_and(|r| !r.is_empty()), "the dialog shows this reason: {resp}");
    assert_eq!(std::fs::read_to_string(root.join("Diagrams/TestDiagram.md")).unwrap(), before);
}

#[tokio::test]
async fn warnings_the_new_diagram_raises_come_back_for_the_dialog() {
    let root = temp_model();
    let app = build_app(&root).await;

    // A part definition is not a valid StateMachine subject: written, drawn
    // empty, W418 (a directory with no `_index.md` is not an element at all, so
    // the fixture's `Basics` would be W401, not W418).
    let (_, wrong_type) = call(
        &app,
        "POST",
        "/api/elements",
        Some(json!({
            "qname": "Diagrams::WrongSubject",
            "type": "Diagram",
            "fields": { "diagramKind": "StateMachine", "subject": "Basics::Widget" }
        })),
    )
    .await;
    assert_eq!(wrong_type["written"], json!(true), "{wrong_type}");
    assert!(warning_codes(&wrong_type).contains(&"W418".to_string()), "{wrong_type}");

    // A subject that resolves to nothing: written, W401.
    let (_, unresolved) = call(
        &app,
        "POST",
        "/api/elements",
        Some(json!({
            "qname": "Diagrams::Dangling",
            "type": "Diagram",
            "fields": { "diagramKind": "IBD", "subject": "Basics::Nope" }
        })),
    )
    .await;
    assert_eq!(unresolved["written"], json!(true), "{unresolved}");
    assert!(warning_codes(&unresolved).contains(&"W401".to_string()), "{unresolved}");
}

#[tokio::test]
async fn a_diagram_in_a_nested_package_lands_under_that_package() {
    let root = temp_model();
    let app = build_app(&root).await;
    let (_, resp) = call(
        &app,
        "POST",
        "/api/elements",
        Some(json!({
            "qname": "Basics::Views::WidgetBDD",
            "type": "Diagram",
            "fields": { "diagramKind": "BDD", "subject": "Basics" }
        })),
    )
    .await;
    assert_eq!(resp["written"], json!(true), "{resp}");
    assert!(root.join("Basics/Views/WidgetBDD.md").exists());
    let (status, graph) = call(&app, "GET", "/api/diagrams/model/Basics/Views/WidgetBDD", None).await;
    assert_eq!(status, StatusCode::OK, "{graph}");
    assert_eq!(graph["derived"], json!(true));
}
