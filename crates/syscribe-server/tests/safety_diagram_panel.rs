//! GH #223: the element detail panel embeds the derived safety diagram of a
//! fault tree, attack tree, safety goal or argument (instead of "No diagram for
//! this element"), each node a link to its element, and
//! `GET /api/diagrams/model/{qname}` serves the same derived graph to the
//! editor. Driven in-process over the shipped `model_auto/` example.

use std::path::{Path, PathBuf};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use syscribe_model::config::ValidateConfig;
use syscribe_model::walker::walk_model;
use syscribe_server::build_router;
use syscribe_server::state::new_state;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../model_auto")
}

async fn get(uri: &str) -> (StatusCode, String) {
    let root = root();
    let elements = walk_model(&root).unwrap();
    let config = ValidateConfig::with_model_root(&root);
    let (shared, reload_tx) = new_state(elements, String::new(), config, root);
    let app = build_router(shared, reload_tx);
    let resp = app.oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap()).await.unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

#[tokio::test]
async fn the_detail_panel_of_a_safety_element_asks_for_its_diagram() {
    for q in ["Safety::FTA::FT-ENG-001", "Security::Attacks::AT-ENG-001", "Safety::HARA::SG-ENG-001", "Safety::Case::ARG-ENG-001"] {
        let (s, html) = get(&format!("/ui/detail/{q}")).await;
        assert_eq!(s, StatusCode::OK);
        assert!(html.contains(&format!("hx-get=\"/ui/diagram/{q}\"")), "{q}: {html}");
    }
    let (_, html) = get("/ui/detail/Safety::FTA").await;
    assert!(!html.contains("/ui/diagram/"), "a package has no safety diagram");
}

#[tokio::test]
async fn the_fragment_embeds_the_svg_with_every_node_linking_to_its_element() {
    let (s, html) = get("/ui/diagram/Safety::FTA::FT-ENG-001").await;
    assert_eq!(s, StatusCode::OK);
    assert!(html.contains("<div class=\"safety-diagram\" data-diagram-kind=\"FaultTree\">") && html.contains("<svg"), "{html}");
    assert!(html.contains("href=\"/ui/detail/Safety::FTA::FT-ENG-001::FTG-ENG-001\""), "node click opens the element: {html}");
    assert!(html.contains("top event"), "the analysis overlay is drawn");
    assert!(!html.contains("No diagram for this element"));

    // A goal has its GSN argument and, since a tree names it, its fault tree.
    let (_, html) = get("/ui/diagram/Safety::HARA::SG-ENG-001").await;
    assert!(html.contains("data-diagram-kind=\"SafetyCase\"") && html.contains("data-diagram-kind=\"FaultTree\""), "{html}");

    let (_, html) = get("/ui/diagram/Security::Attacks::AT-ENG-001").await;
    assert!(html.contains("data-diagram-kind=\"AttackTree\"") && html.contains("feasibility medium"), "{html}");

    // Anything else keeps the old message.
    let (_, html) = get("/ui/diagram/Safety::FTA").await;
    assert!(html.contains("No diagram for this element"), "{html}");
}

#[tokio::test]
async fn the_model_endpoint_serves_the_derived_graph_of_a_safety_element() {
    let (s, body) = get("/api/diagrams/model/Safety::FTA::FT-ENG-001").await;
    assert_eq!(s, StatusCode::OK);
    let j: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(j["diagramKind"], "FaultTree");
    assert_eq!(j["derived"], true);
    assert!(j["children"].as_array().unwrap().iter().any(|c| c["kind"] == "gate-or" || c["kind"] == "gate-and"));

    let (_, body) = get("/api/diagrams/model/Safety::HARA::SG-ENG-001?kind=FaultTree").await;
    let j: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(j["diagramKind"], "FaultTree");
    let (_, body) = get("/api/diagrams/model/Safety::HARA::SG-ENG-001").await;
    let j: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(j["diagramKind"], "SafetyCase", "the primary view of a goal is its argument");

    let (s, _) = get("/api/diagrams/model/Safety::HARA::SG-ENG-001?kind=AttackTree").await;
    assert_eq!(s, StatusCode::NOT_FOUND);
    let (s, _) = get("/api/diagrams/model/Safety::FTA").await;
    assert_eq!(s, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_diagram_element_of_a_safety_kind_is_served_like_any_derived_diagram() {
    let (s, body) = get("/api/diagrams/model/Diagrams::SafetyCaseEngine").await;
    assert_eq!(s, StatusCode::OK);
    let j: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(j["diagramKind"], "SafetyCase");
    assert_eq!(j["qualifiedName"], "Diagrams::SafetyCaseEngine");
}
