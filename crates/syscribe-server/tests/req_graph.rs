//! TC-TRS-REQGRAPH-001 / GH #269 phase 1.

use std::path::Path;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
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

fn req(id: &str, class: &str, extra: &str) -> String {
    format!("---\nid: {id}\ntype: Requirement\nname: {id} name\nstatus: approved\nreqDomain: software\nreqClass: {class}\n{extra}---\n\nShall.\n")
}

fn app(tag: &str) -> axum::Router {
    let root = std::env::temp_dir().join(format!("syscribe-reqgraph-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "R/_index.md", "---\ntype: Package\nname: R\n---\n");
    write(&root, "R/REQ-RG-001.md", &req("REQ-RG-001", "stakeholder", ""));
    write(&root, "R/REQ-RG-002.md", &req("REQ-RG-002", "system", "derivedFrom: [REQ-RG-001]\n"));
    write(&root, "R/REQ-RG-003.md", &req("REQ-RG-003", "system", "derivedFrom: [REQ-RG-002]\n"));
    write(&root, "R/REQ-RG-009.md", &req("REQ-RG-009", "system", ""));
    write(&root, "R/Ctl.md", "---\ntype: PartDef\nname: Ctl\nsatisfies: [REQ-RG-003]\n---\n\nC.\n");
    write(&root, "R/TC-RG-001.md", "---\nid: TC-RG-001\ntype: TestCase\nname: t\nstatus: active\ntestLevel: L3\nverifies: [REQ-RG-003]\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n");
    write(&root, "R/TC-RG-002.md", "---\nid: TC-RG-002\ntype: TestCase\nname: t2\nstatus: draft\ntestLevel: L3\nverifies: [REQ-RG-002]\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n");
    write(&root, "Features/_index.md", "---\ntype: Package\nname: Features\n---\n");
    write(&root, "Features/Opt.md", "---\ntype: FeatureDef\nid: FEAT-OPT-001\nname: Opt\ngroupKind: optional\n---\n\nO.\n");
    write(&root, "R/REQ-RG-004.md", &req("REQ-RG-004", "system", "derivedFrom: [REQ-RG-001]\nappliesWhen: FEAT-OPT-001\n"));
    write(&root, "Configurations/CONF-OFF-001.md", "---\ntype: Configuration\nid: CONF-OFF-001\nname: Off\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Opt: false\n---\n\nOff.\n");
    let elements = walk_model(&root).unwrap();
    let config = ValidateConfig::with_model_root(&root);
    let (shared, tx) = new_state(elements, String::new(), config, root);
    build_router(shared, tx)
}

async fn get(app: &axum::Router, uri: &str) -> (StatusCode, serde_json::Value) {
    let resp = app.clone().oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap()).await.unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null))
}

fn ids(v: &serde_json::Value) -> Vec<String> {
    let mut o: Vec<String> = v["nodes"].as_array().unwrap().iter().map(|n| n["id"].as_str().unwrap().to_string()).collect();
    o.sort();
    o
}

fn has_edge(v: &serde_json::Value, from: &str, to: &str, kind: &str) -> bool {
    v["edges"].as_array().unwrap().iter().any(|e| e["from"] == from && e["to"] == to && e["kind"] == kind)
}

#[tokio::test]
async fn a_neighbourhood_is_bounded_by_depth_and_typed() {
    let a = app("n");
    let (s, v) = get(&a, "/api/req-graph?root=REQ-RG-002").await;
    assert_eq!(s, StatusCode::OK, "{v}");
    assert_eq!(ids(&v), vec!["REQ-RG-001", "REQ-RG-002", "REQ-RG-003", "TC-RG-002"]);
    assert!(has_edge(&v, "REQ-RG-002", "REQ-RG-001", "derivedFrom") && has_edge(&v, "REQ-RG-003", "REQ-RG-002", "derivedFrom"), "{v}");
    assert!(has_edge(&v, "TC-RG-002", "REQ-RG-002", "verifies"));
    let root = v["nodes"].as_array().unwrap().iter().find(|n| n["root"] == true).unwrap();
    assert_eq!(root["id"], "REQ-RG-002");
    // depth 2 reaches the satisfier and the active test of REQ-RG-003
    let (_, v) = get(&a, "/api/req-graph?root=REQ-RG-001&depth=3").await;
    let n = ids(&v);
    assert!(n.contains(&"TC-RG-001".to_string()) && n.contains(&"R::Ctl".to_string()), "{n:?}");
    assert!(has_edge(&v, "R::Ctl", "REQ-RG-003", "satisfies"), "{v}");
}

#[tokio::test]
async fn overlays_filters_and_the_node_cap() {
    let a = app("o");
    let (_, v) = get(&a, "/api/req-graph?root=REQ-RG-003&depth=2").await;
    let node = |id: &str| v["nodes"].as_array().unwrap().iter().find(|n| n["id"] == id).unwrap().clone();
    assert_eq!(node("REQ-RG-003")["verification"], "verified");
    assert_eq!(node("REQ-RG-002")["verification"], "planned");
    assert_eq!(node("REQ-RG-001")["verification"], "unverified");
    assert_eq!(node("TC-RG-001")["verification"], "na");
    assert_eq!(node("REQ-RG-001")["reqClass"], "stakeholder");
    // edge-kind filter
    let (_, f) = get(&a, "/api/req-graph?root=REQ-RG-003&depth=3&edges=verifies").await;
    assert!(f["edges"].as_array().unwrap().iter().all(|e| e["kind"] == "verifies"), "{f}");
    assert_eq!(ids(&f), vec!["REQ-RG-003", "TC-RG-001"]);
    // cap
    let (_, c) = get(&a, "/api/req-graph?root=REQ-RG-003&depth=3&limit=2").await;
    assert_eq!(c["nodes"].as_array().unwrap().len(), 2);
    assert_eq!(c["truncated"], true);
}

#[tokio::test]
async fn configuration_projection_and_errors() {
    let a = app("c");
    let (_, all) = get(&a, "/api/req-graph?root=REQ-RG-001").await;
    assert!(ids(&all).contains(&"REQ-RG-004".to_string()));
    let (s, off) = get(&a, "/api/req-graph?root=REQ-RG-001&config=CONF-OFF-001").await;
    assert_eq!(s, StatusCode::OK, "{off}");
    assert!(!ids(&off).contains(&"REQ-RG-004".to_string()), "{off}");
    assert_eq!(get(&a, "/api/req-graph?root=REQ-NOPE-999").await.0, StatusCode::NOT_FOUND);
    assert_eq!(get(&a, "/api/req-graph").await.0, StatusCode::BAD_REQUEST);
    assert_eq!(get(&a, "/api/req-graph?root=REQ-RG-001&config=CONF-NOPE-001").await.0, StatusCode::BAD_REQUEST);
    assert_eq!(get(&a, "/api/req-graph?root=REQ-RG-001&depth=abc").await.0, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn the_overview_counts_classes_statuses_and_unlinked_requirements() {
    let a = app("v");
    let (s, v) = get(&a, "/api/req-graph/overview").await;
    assert_eq!(s, StatusCode::OK, "{v}");
    assert_eq!(v["requirements"], 5);
    assert_eq!(v["byClass"]["system"], 4);
    assert_eq!(v["byClass"]["stakeholder"], 1);
    assert_eq!(v["byStatus"]["approved"], 5);
    assert_eq!(v["unlinked"], 1, "REQ-RG-009 has no relation: {v}");
    assert_eq!(v["verification"]["verified"], 1);
}
