//! Integration tests for `GET /api/diagrams/model/{*qname}`
//! (`REQ-TRS-VIS-006`, Phase 0 of `ADR-SYS-VIS-001`): the endpoint serves the
//! Diagram IR as a **nested** sprotty `SGraph` — ports inside their block,
//! blocks inside the boundary, edges at the root, ELK `layoutOptions` and the
//! `pinned` set — and `404`s for anything that has no IR.
//!
//! Driven in-process against the real router via `tower::ServiceExt::oneshot`,
//! built exactly as `main` builds it (`build_router` + `new_state`), following
//! `tests/mutate.rs`. The endpoint is read-only, so the checked-in fixture
//! under `tests/fixtures/diagram_model` is served in place — no temp copy.

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
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/diagram_model")
}

async fn build_app() -> axum::Router {
    let root = fixtures_root();
    let elements = walk_model(&root).expect("walk fixture model");
    let config = ValidateConfig::with_model_root(&root);
    let (shared, reload_tx) = new_state(elements, String::new(), config, root);
    build_router(shared, reload_tx)
}

async fn get(app: &axum::Router, uri: &str) -> (StatusCode, Value) {
    let resp = app
        .clone()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .expect("router response");
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

fn child_ids(v: &Value) -> Vec<&str> {
    v["children"]
        .as_array()
        .expect("children array")
        .iter()
        .map(|c| c["id"].as_str().expect("child id"))
        .collect()
}

fn child<'a>(v: &'a Value, id: &str) -> &'a Value {
    v["children"]
        .as_array()
        .expect("children array")
        .iter()
        .find(|c| c["id"] == id)
        .unwrap_or_else(|| panic!("no child `{id}` in {v}"))
}

/// The IBD fixture nests port → block → boundary, keeps edges at the root,
/// pins only the motor, and carries the IBD layout options.
#[tokio::test]
async fn ibd_is_served_nested_with_pins_layout_options_and_root_edges() {
    let app = build_app().await;
    let (status, j) = get(&app, "/api/diagrams/model/Diagrams/SysIBD").await;
    assert_eq!(status, StatusCode::OK);

    assert_eq!(j["id"], "sysml-diagram");
    assert_eq!(j["type"], "graph");
    assert_eq!(j["qualifiedName"], "Diagrams::SysIBD");
    assert_eq!(j["diagramKind"], "IBD");
    assert_eq!(j["subject"], "Sys");

    // Root children: the boundary and the edge, in declaration order.
    assert_eq!(child_ids(&j), vec!["s-sys", "e-flow"]);

    let boundary = child(&j, "s-sys");
    assert_eq!(boundary["type"], "node");
    assert_eq!(boundary["kind"], "boundary");
    assert_eq!(boundary["ref"], "Sys");
    assert_eq!(boundary["resolved"], true);
    assert_eq!(boundary["elementType"], "Package");
    assert!(boundary.get("position").is_none(), "unpinned: no position");
    // Every element carries the server-computed size (REQ-TRS-VIS-017).
    assert!(boundary["size"]["width"].as_f64().unwrap() > 0.0 && boundary["size"]["height"].as_f64().unwrap() > 0.0);
    // The «package» stereotype label precedes the name label (the client's stacking order).
    assert_eq!(child_ids(boundary), vec!["s-sys-stereotype", "s-sys-label", "s-engine", "s-motor", "s-ghost"]);
    assert_eq!(child(boundary, "s-sys-stereotype")["role"], "stereotype");
    let label = child(boundary, "s-sys-label");
    assert_eq!(label["type"], "label");
    assert_eq!(label["text"], "Sys");
    assert_eq!(label["role"], "name");
    assert!(label["size"]["width"].as_f64().unwrap() > 0.0);

    // Block under the boundary, port under the block.
    let engine = child(boundary, "s-engine");
    assert_eq!(engine["type"], "node");
    assert_eq!(engine["kind"], "block");
    assert_eq!(engine["elementType"], "PartDef");
    assert_eq!(engine["stereotype"], "part def");
    assert_eq!(engine["isAbstract"], true);
    assert_eq!(engine["name"], "Engine");
    assert!(engine.get("position").is_none());
    assert_eq!(child_ids(engine), vec!["s-engine-stereotype", "s-engine-label", "s-pout"]);
    assert!(engine["size"]["width"].as_f64().unwrap() >= 120.0, "a leaf block carries at least the client's minimum width");
    let port = child(engine, "s-pout");
    assert_eq!(port["type"], "port");
    assert_eq!(port["kind"], "port");
    assert_eq!(port["direction"], "out");
    assert_eq!(port["stereotype"], "port");
    assert_eq!(port["name"], "powerOut");
    assert_eq!(child_ids(port), vec!["s-pout-label"]);
    // REQ-TRS-VIS-012: resolved style per node/port/edge, and a port side.
    assert_eq!(port["style"], json!({ "fill": "#333", "stroke": "#1f497d", "glyph": "out" }));
    assert_eq!(port["side"], "east", "out → east when the IR has no side");
    assert_eq!(
        engine["style"],
        json!({ "fill": "#f5f5fa", "stroke": "#3a3a4a", "headerFill": null, "text": "#222", "dashed": false })
    );
    assert!(engine.get("banners").is_none(), "no applied stereotypes → no banners key");
    assert_eq!(boundary["style"]["fill"], "#f8f9fb", "a Package boundary falls back to the kind's colours");

    // The pinned motor carries position and size; it is the only pin.
    let motor = child(boundary, "s-motor");
    assert_eq!(motor["position"], json!({ "x": 220.0, "y": 40.0 }));
    assert_eq!(motor["size"], json!({ "width": 150.0, "height": 60.0 }));
    assert_eq!(motor["name"], "Electric Motor");
    assert_eq!(j["pinned"], json!(["s-motor"]));

    // An unresolved ref is drawn (dashed by the client), never dropped.
    let ghost = child(boundary, "s-ghost");
    assert_eq!(ghost["resolved"], false);
    assert_eq!(ghost["name"], "Missing");
    assert!(ghost.get("elementType").is_none());
    assert_eq!(ghost["style"]["dashed"], true);

    // Edges are root children with their kind and pinned waypoints.
    let edge = child(&j, "e-flow");
    assert_eq!(edge["type"], "edge");
    assert_eq!(edge["sourceId"], "s-pout");
    assert_eq!(edge["targetId"], "s-motor");
    assert_eq!(edge["kind"], "flow");
    assert_eq!(edge["routingPoints"], json!([{ "x": 100.0, "y": 50.0 }]));
    assert_eq!(
        edge["style"],
        json!({ "stroke": "#1f497d", "dash": null, "width": 1.4, "arrowTarget": "filled", "arrowSource": "none", "keyword": null })
    );
    for node in [boundary, engine] {
        assert!(node["children"].as_array().unwrap().iter().all(|c| c["type"] != "edge"), "no nested edges");
    }

    assert_eq!(
        j["layoutOptions"],
        json!({
            "elk.algorithm": "layered",
            "elk.direction": "RIGHT",
            "elk.hierarchyHandling": "INCLUDE_CHILDREN",
            "elk.portConstraints": "FIXED_SIDE",
            "syscribe.reversedEdgeKinds": []
        })
    );
}

/// A BDD honours the string shorthand and the map form, reverses inheritance
/// for layering and lays out top-down without hierarchy.
#[tokio::test]
async fn bdd_layout_options_and_shorthand_shapes() {
    let app = build_app().await;
    let (status, j) = get(&app, "/api/diagrams/model/Diagrams/SysBDD").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(j["diagramKind"], "BDD");
    assert_eq!(child_ids(&j), vec!["s-engine", "s-motor", "e-inh"]);
    assert_eq!(child(&j, "s-engine")["kind"], "block");
    assert_eq!(child(&j, "s-engine")["stereotype"], "part def");
    assert_eq!(child(&j, "e-inh")["kind"], "inheritance");
    assert_eq!(child(&j, "e-inh")["style"]["arrowTarget"], "hollowTriangle");
    assert_eq!(child(&j, "e-inh")["style"]["dash"], json!(null));
    assert_eq!(j["layoutOptions"]["elk.direction"], "DOWN");
    assert!(j["layoutOptions"].get("elk.hierarchyHandling").is_none());
    assert_eq!(j["layoutOptions"]["syscribe.reversedEdgeKinds"], json!(["inheritance"]));
    assert_eq!(j["pinned"], json!([]));
}

/// `::` and `/` spellings of the qname both resolve.
#[tokio::test]
async fn qname_accepts_both_separators() {
    let app = build_app().await;
    let (a, ja) = get(&app, "/api/diagrams/model/Diagrams::SysBDD").await;
    let (b, jb) = get(&app, "/api/diagrams/model/Diagrams/SysBDD").await;
    assert_eq!(a, StatusCode::OK);
    assert_eq!(b, StatusCode::OK);
    assert_eq!(ja, jb);
}

/// A `subject:`-only diagram is derived from the model (`REQ-TRS-VIS-003`):
/// an IBD of a bare `PartDef` is its boundary alone.
#[tokio::test]
async fn derived_diagram_returns_the_generated_graph_for_its_subject() {
    let app = build_app().await;
    let (status, j) = get(&app, "/api/diagrams/model/Diagrams/Derived").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(j["subject"], "Sys::Engine");
    assert_eq!(j["diagramKind"], "IBD");
    let children = j["children"].as_array().unwrap();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0]["id"], "s-sys-engine");
    assert_eq!(children[0]["kind"], "boundary");
    assert_eq!(children[0]["ref"], "Sys::Engine");
    assert_eq!(children[0]["isAbstract"], json!(true));
    assert_eq!(j["pinned"], json!([]));
}

/// Mermaid diagrams, non-diagram elements and unknown names have no sprotty model.
#[tokio::test]
async fn mermaid_non_diagram_and_unknown_are_404() {
    let app = build_app().await;
    for uri in [
        "/api/diagrams/model/Diagrams/Flow",
        "/api/diagrams/model/Sys/Engine",
        "/api/diagrams/model/Diagrams/Nope",
    ] {
        let (status, _) = get(&app, uri).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{uri}");
    }
}
