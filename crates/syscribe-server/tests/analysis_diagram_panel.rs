//! GH #223 (phase 2): the element detail panel embeds the derived analysis
//! graphs — hazard-to-test traceability on a hazardous event, safety goal and
//! requirement that traces to a goal; the zone/conduit picture on a zone or
//! conduit; the threat graph on a threat, damage scenario, asset, goal or
//! control — and `GET /api/diagrams/model/{qname}` serves the same graphs to the
//! editor, with the project's `[cyber]` configuration behind the risk tones.
//! Driven in-process over the shipped `model_auto/` example.

use std::path::{Path, PathBuf};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use syscribe_model::config::ValidateConfig;
use syscribe_model::walker::walk_model;
use syscribe_server::build_router;
use syscribe_server::state::new_state;

fn auto() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../model_auto")
}

async fn get_in(root: PathBuf, uri: &str) -> (StatusCode, String) {
    let elements = walk_model(&root).unwrap();
    let config = ValidateConfig::with_model_root(&root);
    let (shared, reload_tx) = new_state(elements, String::new(), config, root);
    let app = build_router(shared, reload_tx);
    let resp = app.oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap()).await.unwrap();
    let status = resp.status();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

async fn get(uri: &str) -> (StatusCode, String) {
    get_in(auto(), uri).await
}

#[tokio::test]
async fn the_detail_panel_asks_for_the_analysis_graph_of_the_right_elements() {
    for q in [
        "Safety::HARA::HE-ENG-001",
        "Safety::HARA::SG-ENG-001",
        "Requirements::Safety::REQ-ENG-SAFE-001",
        "Security::Zones::ZN-ENG-001",
        "Security::Zones::CD-ENG-001",
        "Security::TARA-ENG-001::TS-ENG-001",
        "Security::TARA-ENG-001::DS-ENG-001",
        "Security::Assets::ASSET-ENG-001",
        "Security::TARA-ENG-001::CSG-ENG-001",
        "Security::TARA-ENG-001::SC-ENG-001",
    ] {
        let (s, html) = get(&format!("/ui/detail/{q}")).await;
        assert_eq!(s, StatusCode::OK, "{q}");
        assert!(html.contains(&format!("hx-get=\"/ui/diagram/{q}\"")), "{q}: {html}");
    }
    // A plain part and a package have none.
    for q in ["System::EngineECU", "Security::Zones"] {
        let (_, html) = get(&format!("/ui/detail/{q}")).await;
        assert!(!html.contains("/ui/diagram/"), "{q}");
    }
}

#[tokio::test]
async fn the_fragment_embeds_each_graph_as_svg_with_click_through_nodes() {
    let (s, html) = get("/ui/diagram/Safety::HARA::SG-ENG-001").await;
    assert_eq!(s, StatusCode::OK);
    // The safety views come first, the traceability graph after.
    let order: Vec<usize> = ["SafetyCase", "FaultTree", "Traceability"].iter().map(|k| html.find(&format!("data-diagram-kind=\"{k}\"")).unwrap_or(usize::MAX)).collect();
    assert!(order[0] < order[1] && order[1] < order[2], "{order:?}");
    assert!(html.contains("Hazard-to-test traceability"), "{html}");
    assert!(html.contains("href=\"/ui/detail/Requirements::Safety::REQ-ENG-SAFE-001\""), "a node opens its element: {html}");

    let (_, html) = get("/ui/diagram/Security::Zones::ZN-ENG-001").await;
    assert!(html.contains("data-diagram-kind=\"ZoneConduit\"") && html.contains("Powertrain Control Zone") && html.contains("SL target 3 / achieved 3"), "{html}");
    assert!(html.contains("href=\"/ui/detail/System::Software::SafetyMonitor\""), "{html}");

    let (_, html) = get("/ui/diagram/Security::TARA-ENG-001::CSG-ENG-001").await;
    assert!(html.contains("data-diagram-kind=\"ThreatGraph\"") && html.contains("risk critical"), "{html}");
    assert!(html.contains("href=\"/ui/detail/Security::TARA-ENG-001::SC-ENG-001\""), "{html}");
}

#[tokio::test]
async fn the_model_endpoint_serves_the_analysis_graphs_to_the_editor() {
    let (s, body) = get("/api/diagrams/model/Security::Zones::ZN-ENG-002").await;
    assert_eq!(s, StatusCode::OK);
    let j: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(j["diagramKind"], "ZoneConduit");
    assert_eq!(j["derived"], true);
    let kids = j["children"].as_array().unwrap();
    let zone = kids.iter().find(|c| c["kind"] == "zone").expect("a zone node");
    assert!(zone["children"].as_array().unwrap().iter().any(|c| c["kind"] == "node" || c["type"] == "node"), "the zone holds its parts: {zone}");
    assert!(zone["mark"]["status"].as_str().unwrap().starts_with("SL target"));

    // The primary view of a goal stays its argument; ?kind= picks the traceability graph.
    let (_, body) = get("/api/diagrams/model/Safety::HARA::SG-ENG-001").await;
    assert_eq!(serde_json::from_str::<serde_json::Value>(&body).unwrap()["diagramKind"], "SafetyCase");
    let (_, body) = get("/api/diagrams/model/Safety::HARA::SG-ENG-001?kind=Traceability").await;
    let j: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(j["diagramKind"], "Traceability");
    assert!(j["children"].as_array().unwrap().iter().any(|c| c["kind"] == "requirement"));

    // A threat that an attack tree substantiates has that tree as its primary view; the threat graph is ?kind=.
    let (_, body) = get("/api/diagrams/model/Security::TARA-ENG-001::TS-ENG-001").await;
    assert_eq!(serde_json::from_str::<serde_json::Value>(&body).unwrap()["diagramKind"], "AttackTree");
    let (_, body) = get("/api/diagrams/model/Security::TARA-ENG-001::TS-ENG-001?kind=ThreatGraph").await;
    assert_eq!(serde_json::from_str::<serde_json::Value>(&body).unwrap()["diagramKind"], "ThreatGraph");
    let (_, body) = get("/api/diagrams/model/Security::TARA-ENG-001::CSG-ENG-001").await;
    assert_eq!(serde_json::from_str::<serde_json::Value>(&body).unwrap()["diagramKind"], "ThreatGraph");

    // A kind the element does not support, and a part, are 404.
    assert_eq!(get("/api/diagrams/model/Security::Zones::ZN-ENG-002?kind=ThreatGraph").await.0, StatusCode::NOT_FOUND);
    assert_eq!(get("/api/diagrams/model/System::EngineECU").await.0, StatusCode::NOT_FOUND);

    // The Diagram elements shipped with the example are served like any derived diagram.
    for (q, kind) in [("TraceabilityEngine", "Traceability"), ("ZoneConduitEngine", "ZoneConduit"), ("ThreatGraphEngine", "ThreatGraph")] {
        let (s, body) = get(&format!("/api/diagrams/model/Diagrams::{q}")).await;
        assert_eq!(s, StatusCode::OK, "{q}");
        assert_eq!(serde_json::from_str::<serde_json::Value>(&body).unwrap()["diagramKind"], kind);
    }
}

#[tokio::test]
async fn the_threat_graph_uses_the_projects_cyber_configuration() {
    // The same model, with a risk-matrix override: the threat's tone follows it.
    let root = std::env::temp_dir().join(format!("syscribe-analysis-panel-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("Sec")).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(root.join("Sec/_index.md"), "---\ntype: Package\nname: Sec\n---\n").unwrap();
    std::fs::write(
        root.join("Sec/TARA.md"),
        "---\ntype: TARASheet\nid: TARA-PN-001\nname: TARA\nstatus: approved\n\
damageTable:\n  - id: DS-PN-001\n    name: Hijack\n    damageSeverity: negligible\n\
threatTable:\n  - id: TS-PN-001\n    name: Spoof\n    attackFeasibility: high\n    damageScenarios: [DS-PN-001]\n---\n\nT.\n",
    )
    .unwrap();
    let (s, plain) = get_in(root.clone(), "/api/diagrams/model/Sec::TARA::TS-PN-001").await;
    assert_eq!(s, StatusCode::OK);
    assert!(plain.contains("risk medium"), "{plain}");
    std::fs::write(root.join(".syscribe.toml"), "[cyber.risk_matrix.negligible]\nhigh = \"critical\"\n").unwrap();
    let (_, configured) = get_in(root.clone(), "/api/diagrams/model/Sec::TARA::TS-PN-001").await;
    assert!(configured.contains("risk critical"), "{configured}");
    let (_, html) = get_in(root.clone(), "/ui/diagram/Sec::TARA::TS-PN-001").await;
    assert!(html.contains("risk critical"), "{html}");
    let _ = std::fs::remove_dir_all(&root);
}
