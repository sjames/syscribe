//! `REQ-TRS-MCP-MEM-000`: a guarded write on a large model drops the live copy while it builds
//! the candidate and rebuilds afterwards. The threshold is lowered here so a handful of elements
//! takes the large-model path.

use std::path::Path;

use axum::body::Body;
use axum::http::Request;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

use syscribe_model::config::ValidateConfig;
use syscribe_model::walker::walk_model;
use syscribe_server::build_router;
use syscribe_server::state::new_state;

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

async fn call(app: &axum::Router, method: &str, uri: &str, body: Option<Value>) -> Value {
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

#[tokio::test]
async fn a_write_on_a_large_model_leaves_it_whole_and_keeps_findings_between_writes() {
    let root = std::env::temp_dir().join(format!("syscribe-release-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    for i in 0..6 {
        write(&root, &format!("P/Part{i}.md"), &format!("---\ntype: PartDef\nname: Part{i}\n---\n\nPart {i}.\n"));
    }
    let elements = walk_model(&root).unwrap();
    let count = elements.len();
    let config = ValidateConfig::with_model_root(&root);
    let (shared, reload_tx) = new_state(elements, String::new(), config, root.clone());
    shared.write().await.release_above = 1; // every model is "large"
    let app = build_router(shared.clone(), reload_tx);

    // A dry run drops the model while the candidate is built and must put it back.
    let dry = call(&app, "POST", "/api/elements", Some(json!({"qname": "P::Dry", "type": "PartDef", "dryRun": true}))).await;
    assert_eq!(dry["written"], false, "{dry}");
    assert_eq!(shared.read().await.elements.len(), count, "the model is back after a dry run");
    let listed = call(&app, "GET", "/api/elements?type=PartDef", None).await;
    assert_eq!(listed.as_array().unwrap().len(), 6);

    // A commit reloads, and the next write's delta names only its own file.
    let first = call(&app, "POST", "/api/elements", Some(json!({"qname": "P::Added0", "type": "PartDef"}))).await;
    assert_eq!(first["written"], true, "{first}");
    assert!(shared.read().await.baseline.is_some(), "the candidate's findings are kept as the next baseline");
    let second = call(&app, "POST", "/api/elements", Some(json!({"qname": "P::Added1", "type": "PartDef"}))).await;
    assert_eq!(second["written"], true, "{second}");
    let delta = serde_json::to_string(&second["newWarnings"]).unwrap() + &serde_json::to_string(&second["newErrors"]).unwrap();
    assert!(!delta.contains("Added0"), "the second write repeats the first's findings: {delta}");
    assert_eq!(shared.read().await.elements.len(), count + 2);

    // A refused write rebuilds too.
    let dup = call(&app, "POST", "/api/elements", Some(json!({"qname": "P::Added1", "type": "PartDef"}))).await;
    assert_eq!(dup["written"], false);
    assert_eq!(shared.read().await.elements.len(), count + 2);
}
