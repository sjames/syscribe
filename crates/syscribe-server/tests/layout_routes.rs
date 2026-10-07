//! Integration tests for the diagram layout and companion-SVG routes
//! (`REQ-TRS-VIS-006` second paragraph, `REQ-TRS-VIS-011`) in `routes::mutate`:
//!
//! - `PATCH /api/diagrams/layout/{*qname}` writes pins (`x`/`y`, and `w`/`h`
//!   when given); a `null` value removes one pin and leaves the others;
//! - `DELETE /api/diagrams/layout/{*qname}` removes the whole `layout:` key;
//! - `PUT /api/diagrams/svg/{*qname}` writes the companion file, sets
//!   `svgMode`/`svgFile` when absent and appends one `<img>` (never twice);
//!   a body that is not an SVG document is refused and writes nothing.
//!
//! Same in-process pattern as `tests/mutate.rs`: the shared
//! `tests/fixtures/mutate` model is copied into a fresh temp directory per test.

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
            "syscribe-server-layout-test-{}-{}-{}",
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

/// A two-shape diagram with both shapes pinned, replacing the fixture's
/// one-shape `TestDiagram` so the "remove one pin, keep the other" case has
/// something to keep.
const TWO_PIN_DIAGRAM: &str = "---
type: Diagram
name: TestDiagram
diagramKind: BDD
shapes:
  s-widget:
    ref: Basics::Widget
    kind: PartDef
  s-widget2:
    ref: Basics::Widget
    kind: PartDef
layout:
  s-widget:
    x: 10
    y: 10
  s-widget2:
    x: 200
    y: 10
---

Two pinned shapes.
";

fn diagram_path(root: &Path) -> PathBuf {
    root.join("Diagrams/TestDiagram.md")
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

/// The `layout:` map of the diagram on disk, re-parsed (`None` when the key is absent).
fn layout_on_disk(root: &Path) -> Option<serde_yaml::Mapping> {
    let content = std::fs::read_to_string(diagram_path(root)).unwrap();
    let (fm, _) = syscribe_model::frontmatter::split_frontmatter(&content);
    let v: serde_yaml::Value = serde_yaml::from_str(fm.expect("frontmatter")).unwrap();
    match v.get("layout") {
        None => None,
        Some(serde_yaml::Value::Mapping(m)) => Some(m.clone()),
        Some(other) => panic!("layout is not a map: {other:?}"),
    }
}

fn num(m: &serde_yaml::Mapping, key: &str) -> Option<i64> {
    m.get(key).and_then(|v| v.as_i64())
}

/// A `{x, y, w, h}` value writes all four; `w`/`h` were never written before.
#[tokio::test]
async fn patch_with_w_and_h_writes_them() {
    let model_root = temp_model();
    let app = build_app(&model_root).await;

    let (status, resp) = call(
        &app,
        "PATCH",
        "/api/diagrams/layout/Diagrams/TestDiagram",
        Some(json!({ "s-widget": { "x": 30.4, "y": 40.6, "w": 150.0, "h": 60.0 } })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["written"], json!(true), "expected commit, got {resp:#?}");

    let layout = layout_on_disk(&model_root).expect("layout present");
    let pin = layout.get("s-widget").and_then(|v| v.as_mapping()).expect("s-widget pin");
    assert_eq!((num(pin, "x"), num(pin, "y"), num(pin, "w"), num(pin, "h")), (Some(30), Some(41), Some(150), Some(60)));

    // A later x/y-only patch keeps the size.
    let (_, resp) = call(
        &app,
        "PATCH",
        "/api/diagrams/layout/Diagrams/TestDiagram",
        Some(json!({ "s-widget": { "x": 1, "y": 2 } })),
    )
    .await;
    assert_eq!(resp["written"], json!(true), "{resp:#?}");
    let layout = layout_on_disk(&model_root).unwrap();
    let pin = layout.get("s-widget").and_then(|v| v.as_mapping()).unwrap();
    assert_eq!((num(pin, "x"), num(pin, "y"), num(pin, "w"), num(pin, "h")), (Some(1), Some(2), Some(150), Some(60)));

    let _ = std::fs::remove_dir_all(&model_root);
}

/// A `null` value removes that shape's pin and leaves the others untouched.
#[tokio::test]
async fn patch_null_removes_one_pin_and_leaves_others() {
    let model_root = temp_model();
    std::fs::write(diagram_path(&model_root), TWO_PIN_DIAGRAM).unwrap();
    let app = build_app(&model_root).await;

    let (status, resp) = call(
        &app,
        "PATCH",
        "/api/diagrams/layout/Diagrams/TestDiagram",
        Some(json!({ "s-widget": null })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["written"], json!(true), "expected commit, got {resp:#?}");

    let layout = layout_on_disk(&model_root).expect("layout key stays while a pin remains");
    assert!(!layout.contains_key("s-widget"), "pin removed: {layout:?}");
    let other = layout.get("s-widget2").and_then(|v| v.as_mapping()).expect("other pin kept");
    assert_eq!((num(other, "x"), num(other, "y")), (Some(200), Some(10)));

    // Unpinning a shape that has no pin is a harmless no-op write.
    let (_, resp) = call(
        &app,
        "PATCH",
        "/api/diagrams/layout/Diagrams/TestDiagram",
        Some(json!({ "s-widget": null, "s-widget2": { "x": 7, "y": 8 } })),
    )
    .await;
    assert_eq!(resp["written"], json!(true), "{resp:#?}");
    let layout = layout_on_disk(&model_root).unwrap();
    assert_eq!(layout.len(), 1);
    assert_eq!(num(layout.get("s-widget2").and_then(|v| v.as_mapping()).unwrap(), "x"), Some(7));

    let _ = std::fs::remove_dir_all(&model_root);
}

/// `DELETE` drops the whole `layout:` key (every pin); the shapes stay.
#[tokio::test]
async fn delete_removes_the_layout_key() {
    let model_root = temp_model();
    std::fs::write(diagram_path(&model_root), TWO_PIN_DIAGRAM).unwrap();
    let app = build_app(&model_root).await;

    let (status, resp) = call(&app, "DELETE", "/api/diagrams/layout/Diagrams/TestDiagram", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["written"], json!(true), "expected commit, got {resp:#?}");
    assert!(resp.get("newErrors").is_some(), "WriteResponse shape");

    assert!(layout_on_disk(&model_root).is_none(), "`layout:` key must be gone");
    let content = std::fs::read_to_string(diagram_path(&model_root)).unwrap();
    assert!(content.contains("s-widget2:"), "shapes stay:\n{content}");
    assert!(content.contains("Two pinned shapes."), "body stays:\n{content}");

    // The sprotty model now has no pins.
    let (_, j) = call(&app, "GET", "/api/diagrams/model/Diagrams/TestDiagram", None).await;
    assert_eq!(j["pinned"], json!([]));

    // Unknown qname and non-Diagram elements are refused with a reason.
    let (status, resp) = call(&app, "DELETE", "/api/diagrams/layout/Nonexistent/Ghost", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["written"], json!(false));
    assert!(resp["reason"].as_str().unwrap().contains("unresolved"), "{resp:#?}");
    let (_, resp) = call(&app, "DELETE", "/api/diagrams/layout/Basics/Widget", None).await;
    assert_eq!(resp["written"], json!(false));
    assert!(resp["reason"].as_str().unwrap().contains("not a Diagram"), "{resp:#?}");

    let _ = std::fs::remove_dir_all(&model_root);
}

/// `PUT svg` writes the companion file, sets `svgMode: companion` and
/// `svgFile: ./<stem>.svg`, appends the `<img>` once — and a second PUT
/// overwrites the file without appending again.
#[tokio::test]
async fn put_svg_writes_companion_sets_fields_and_appends_img_once() {
    let model_root = temp_model();
    let app = build_app(&model_root).await;
    let svg_path = model_root.join("Diagrams/TestDiagram.svg");
    assert!(!svg_path.exists());

    let first = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"10\" height=\"10\"><rect sysml:ref=\"Basics::Widget\"/></svg>";
    let (status, resp) = call(
        &app,
        "PUT",
        "/api/diagrams/svg/Diagrams/TestDiagram",
        Some(json!({ "svg": format!("  \n{first}\n") })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resp["written"], json!(true), "expected commit, got {resp:#?}");
    assert!(resp.get("newErrors").is_some(), "WriteResponse shape");

    assert_eq!(std::fs::read_to_string(&svg_path).unwrap(), format!("{first}\n"));
    let content = std::fs::read_to_string(diagram_path(&model_root)).unwrap();
    assert!(content.contains("svgMode: companion"), "{content}");
    assert!(content.contains("svgFile: ./TestDiagram.svg"), "{content}");
    assert_eq!(content.matches("<img").count(), 1, "{content}");
    assert!(content.contains("<img src=\"./TestDiagram.svg\" alt=\"TestDiagram\" width=\"100%\"/>"), "{content}");
    assert!(content.contains("verify shape/layout/edge sync"), "original body kept:\n{content}");
    // No E402/W405 appeared: the companion exists and the body has the img.
    assert!(resp["newErrors"].as_array().unwrap().is_empty(), "{resp:#?}");
    assert!(
        !resp["newWarnings"].as_array().unwrap().iter().any(|w| w["code"] == "W405"),
        "{resp:#?}"
    );

    let second = "<svg xmlns=\"http://www.w3.org/2000/svg\"><g/></svg>";
    let (_, resp) = call(
        &app,
        "PUT",
        "/api/diagrams/svg/Diagrams/TestDiagram",
        Some(json!({ "svg": second })),
    )
    .await;
    assert_eq!(resp["written"], json!(true), "{resp:#?}");
    assert_eq!(std::fs::read_to_string(&svg_path).unwrap(), format!("{second}\n"));
    let content = std::fs::read_to_string(diagram_path(&model_root)).unwrap();
    assert_eq!(content.matches("<img").count(), 1, "img appended once only:\n{content}");
    assert_eq!(content.matches("svgMode:").count(), 1);
    assert_eq!(content.matches("svgFile:").count(), 1);

    let _ = std::fs::remove_dir_all(&model_root);
}

/// An explicit `svgFile:` is honoured (relative to the `.md`) and left as is.
#[tokio::test]
async fn put_svg_honours_an_existing_svg_file_field() {
    let model_root = temp_model();
    std::fs::write(
        diagram_path(&model_root),
        "---\ntype: Diagram\nname: TestDiagram\ndiagramKind: BDD\nsvgFile: pictures/custom.svg\nshapes:\n  s-widget: Basics::Widget\n---\n\nBody.\n",
    )
    .unwrap();
    let app = build_app(&model_root).await;

    let (_, resp) = call(
        &app,
        "PUT",
        "/api/diagrams/svg/Diagrams/TestDiagram",
        Some(json!({ "svg": "<svg xmlns=\"http://www.w3.org/2000/svg\"/></svg>" })),
    )
    .await;
    assert_eq!(resp["written"], json!(true), "{resp:#?}");
    assert!(model_root.join("Diagrams/pictures/custom.svg").exists());
    assert!(!model_root.join("Diagrams/TestDiagram.svg").exists());
    let content = std::fs::read_to_string(diagram_path(&model_root)).unwrap();
    assert_eq!(content.matches("svgFile:").count(), 1);
    assert!(content.contains("svgFile: pictures/custom.svg"), "{content}");
    assert!(content.contains("svgMode: companion"), "{content}");
    assert!(content.contains("<img src=\"pictures/custom.svg\""), "{content}");

    let _ = std::fs::remove_dir_all(&model_root);
}

/// A body that is not an SVG document is refused: no file, `.md` untouched.
#[tokio::test]
async fn put_svg_refuses_non_svg_and_writes_nothing() {
    let model_root = temp_model();
    let app = build_app(&model_root).await;
    let before = std::fs::read_to_string(diagram_path(&model_root)).unwrap();

    for bad in ["<div>not svg</div>", "<svg unterminated", "", "svg"] {
        let (status, resp) = call(
            &app,
            "PUT",
            "/api/diagrams/svg/Diagrams/TestDiagram",
            Some(json!({ "svg": bad })),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(resp["written"], json!(false), "{bad:?} must be refused: {resp:#?}");
        assert!(resp["reason"].as_str().unwrap().contains("SVG"), "{resp:#?}");
    }
    assert!(!model_root.join("Diagrams/TestDiagram.svg").exists());
    assert_eq!(std::fs::read_to_string(diagram_path(&model_root)).unwrap(), before);

    // A non-Diagram target is refused too.
    let (_, resp) = call(
        &app,
        "PUT",
        "/api/diagrams/svg/Basics/Widget",
        Some(json!({ "svg": "<svg></svg>" })),
    )
    .await;
    assert_eq!(resp["written"], json!(false));
    assert!(resp["reason"].as_str().unwrap().contains("not a Diagram"), "{resp:#?}");
    assert!(!model_root.join("Basics/Widget.svg").exists());

    let _ = std::fs::remove_dir_all(&model_root);
}
