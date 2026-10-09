//! GH #223 (phase 2): the hazard-to-test, zone/conduit and threat graphs from
//! the command line — `diagram export`, `zones --format`, `cyber-risk --format
//! dot|mermaid`, `hara trace`, and `connectivity` drawing the safety / security
//! links — plus the `zones --coverage` fix. Driven over the shipped `model_auto/`
//! example and small temp models.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

fn auto() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../model_auto")
}

fn run_in(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().expect("spawn syscribe")
}

fn out_in(root: &Path, args: &[&str]) -> String {
    let o = run_in(root, args);
    assert!(o.status.success(), "{args:?}: {}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn out(args: &[&str]) -> String {
    out_in(&auto(), args)
}

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn temp_model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-analysis-cli-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed))).join("model");
    std::fs::create_dir_all(&root).unwrap();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    root
}

#[test]
fn diagram_export_writes_the_three_analysis_kinds_in_every_format() {
    for (diagram, marker) in [
        ("Diagrams::TraceabilityEngine", "ASIL D"),
        ("Diagrams::ZoneConduitEngine", "SL target 3 / achieved 3"),
        ("Diagrams::ThreatGraphEngine", "risk critical"),
    ] {
        let dot = out(&["diagram", "export", diagram, "--format", "dot"]);
        assert!(dot.starts_with("digraph") && dot.contains(marker), "{diagram}: {dot}");
        let mm = out(&["diagram", "export", diagram, "--format", "mermaid"]);
        assert!(mm.starts_with("flowchart LR") && mm.contains(marker), "{diagram}: {mm}");
        let svg = out(&["diagram", "export", diagram, "--format", "svg"]);
        assert!(svg.starts_with("<svg") && svg.contains(marker), "{diagram}");
        let puml = out(&["diagram", "export", diagram, "--format", "plantuml"]);
        assert!(puml.starts_with("@startuml") && puml.contains(marker), "{diagram}: {puml}");
    }
    // The zone diagram is Graphviz-valid with clusters.
    let dot = out(&["diagram", "export", "Diagrams::ZoneConduitEngine", "--format", "dot"]);
    assert!(dot.contains("subgraph cluster_") && dot.contains("lhead="), "{dot}");
}

#[test]
fn threat_graph_export_follows_the_configured_risk_method() {
    let default = out(&["diagram", "export", "Diagrams::ThreatGraphEngine", "--format", "dot"]);
    let root = temp_model();
    // A model of its own with the override: copy the one threat chain and configure its cell.
    write(&root, ".syscribe.toml", "[cyber.risk_matrix.negligible]\nhigh = \"critical\"\n");
    write(
        &root,
        "Sec/TARA.md",
        "---\ntype: TARASheet\nid: TARA-CF-001\nname: TARA\nstatus: approved\n\
damageTable:\n  - id: DS-CF-001\n    name: Hijack\n    damageSeverity: negligible\n\
threatTable:\n  - id: TS-CF-001\n    name: Spoof\n    attackFeasibility: high\n    damageScenarios: [DS-CF-001]\n---\n\nT.\n",
    );
    write(&root, "Diagrams/Tg.md", "---\ntype: Diagram\nname: Tg\ndiagramKind: ThreatGraph\nsubject: Sec::TARA\n---\n\nT.\n");
    let configured = out_in(&root, &["diagram", "export", "Diagrams::Tg", "--format", "dot"]);
    assert!(configured.contains("risk critical"), "the [cyber] override reaches the diagram: {configured}");
    root.parent().map(std::fs::remove_dir_all);
    assert!(default.contains("risk critical") && default.contains("risk medium"), "model_auto uses the default method");
}

#[test]
fn zones_format_draws_the_whole_model_and_the_default_table_is_unchanged() {
    let table = out(&["zones"]);
    assert!(table.starts_with("| Zone | tSL | aSL | Members | Gap |"), "{table}");
    assert!(table.contains("| ZN-ENG-001 | 3 | 3 | 3 | ✓ |"), "{table}");

    let mm = out(&["zones", "--format", "mermaid"]);
    assert!(mm.starts_with("flowchart LR") && mm.contains("subgraph") && mm.contains("CD-ENG-001 · SL 3/3 · CAN"), "{mm}");
    let dot = out(&["zones", "--format", "dot"]);
    assert!(dot.contains("cluster_") && dot.contains("Powertrain Control Zone"), "{dot}");
    assert!(out(&["zones", "--format", "svg"]).starts_with("<svg"));
    assert!(out(&["zones", "--format", "plantuml"]).starts_with("@startuml ZoneConduit"));

    let bad = run_in(&auto(), &["zones", "--format", "pdf"]);
    assert!(!bad.status.success());
    assert!(String::from_utf8_lossy(&bad.stderr).contains("valid values: dot, mermaid, plantuml, svg"));
    // A model with no zone has nothing to draw.
    let empty = run_in(&temp_model(), &["zones", "--format", "dot"]);
    assert!(!empty.status.success());
}

/// GH #223: `zones --coverage` showed `—` in every control column because it
/// looked for a SecurityControl among a zone's `members:` (which must be parts,
/// `E955`) and in a conduit's `implementedBy:`, never at the allocation that
/// actually binds a control to the architecture.
#[test]
fn zones_coverage_lists_controls_allocated_to_a_zones_parts() {
    let cov = out(&["zones", "--coverage"]);
    assert!(cov.contains("| ZN-ENG-002 | 2 | SC-ENG-002, SC-ENG-003, SC-ENG-004 |"), "{cov}");
    assert!(cov.contains("| ZN-ENG-003 | 2 | SC-ENG-001 |"), "{cov}");
    assert!(cov.contains("| ZN-ENG-001 | 3 | — |"), "a zone with no allocated control still shows the gap: {cov}");
    let json: serde_json::Value = serde_json::from_str(&out(&["zones", "--coverage", "--json"])).unwrap();
    let z2 = json["coverage"].as_array().unwrap().iter().find(|z| z["zone"] == "ZN-ENG-002").unwrap();
    assert_eq!(z2["controls"], serde_json::json!(["SC-ENG-002", "SC-ENG-003", "SC-ENG-004"]));
}

#[test]
fn conduits_and_zone_json_agree_with_the_shared_logic() {
    let root = temp_model();
    write(&root, "Sec/_index.md", "---\ntype: Package\nname: Sec\n---\n");
    write(&root, "Sec/Z1.md", "---\ntype: Zone\nid: ZN-XX-001\nname: A\nstatus: approved\ntargetSL: 3\nachievedSL: 3\n---\n\nz\n");
    write(&root, "Sec/Z2.md", "---\ntype: Zone\nid: ZN-XX-002\nname: B\nstatus: approved\ntargetSL: 2\nachievedSL: 1\n---\n\nz\n");
    write(&root, "Sec/C1.md", "---\ntype: Conduit\nid: CD-XX-001\nname: weak\nstatus: approved\nfromZone: ZN-XX-002\ntoZone: ZN-XX-001\nachievedSL: 2\n---\n\nc\n");
    let table = out_in(&root, &["conduits"]);
    assert!(table.contains("⚠ weak"), "{table}");
    let json: serde_json::Value = serde_json::from_str(&out_in(&root, &["conduits", "--json"])).unwrap();
    assert_eq!(json["conduits"][0]["pass"], false);
    assert_eq!(json["conduits"][0]["requiredSL"], 3);
    let zones = out_in(&root, &["zones"]);
    assert!(zones.contains("⚠ SL gap"), "{zones}");
    // The picture marks the same conduit as weak.
    let dot = out_in(&root, &["zones", "--format", "dot"]);
    assert!(dot.contains("weak") && dot.contains("penwidth=3"), "{dot}");
    root.parent().map(std::fs::remove_dir_all);
}

#[test]
fn cyber_risk_format_dot_and_mermaid_draw_the_threat_graph_and_heat_formats_stay() {
    let dot = out(&["cyber-risk", "--format", "dot"]);
    assert!(dot.starts_with("digraph") && dot.contains("risk critical") && dot.contains("protectedBy"), "{dot}");
    let mm = out(&["cyber-risk", "--format", "mermaid"]);
    assert!(mm.starts_with("flowchart LR") && mm.contains("classDef tone_bad"), "{mm}");
    // The heat tables are still md | html | json.
    assert!(out(&["cyber-risk", "--format", "md"]).contains("risk matrix") || out(&["cyber-risk", "--format", "md"]).contains("Risk"));
    assert!(out(&["cyber-risk", "--format", "json"]).trim_start().starts_with('{'));
    assert!(!run_in(&auto(), &["cyber-risk", "--format", "pdf"]).status.success());
}

#[test]
fn hara_trace_draws_the_hazard_to_test_graph_with_ingested_verdicts() {
    let mm = out(&["hara", "trace", "SG-ENG-001", "--format", "mermaid"]);
    assert!(mm.starts_with("flowchart LR") && mm.contains("unknown"), "no sidecar: unknown verdicts: {mm}");
    let dot = out(&["hara", "trace", "--format", "dot"]);
    assert!(dot.contains("HIL — safety monitor") || dot.contains("REQ-ENG-SAFE"), "whole model by default: {dot}");
    assert!(!run_in(&auto(), &["hara", "trace", "TC-ENG-SAFE-002"]).status.success(), "a test case is not a subject");
    assert!(!run_in(&auto(), &["hara", "trace", "--format", "pdf"]).status.success());

    // With a results sidecar the verdicts are real: a failing test turns the chain red.
    let root = temp_model();
    write(&root, "Safety/SG.md", "---\ntype: SafetyGoal\nid: SG-CL-001\nname: Goal\nstatus: approved\nasilLevel: B\nsafeState: s\n---\n\nG.\n");
    write(
        &root,
        "Safety/REQ.md",
        "---\ntype: Requirement\nid: REQ-CL-001\nname: Req\nstatus: approved\nreqDomain: system\nderivedFromSafetyGoal: SG-CL-001\n---\n\nShall.\n",
    );
    write(
        &root,
        "Safety/TC.md",
        "---\ntype: TestCase\nid: TC-CL-001\nname: Test\nstatus: active\ntestLevel: L3\nverifies: [REQ-CL-001]\ntestFunctions:\n  - function: tests::chain\n---\n\nT.\n",
    );
    let before = out_in(&root, &["hara", "trace", "SG-CL-001", "--format", "dot"]);
    assert!(before.contains("unknown"), "{before}");
    // The sidecar `ingest-results` writes: a failing test function turns the chain red.
    write(
        &root,
        ".syscribe/results.json",
        "{\"schema_version\":\"1\",\"format\":\"cargo-json\",\"source\":\"t\",\"ingested_at_unix\":0,\"count\":1,\"by_leaf\":{\"chain\":\"fail\"}}",
    );
    let after = out_in(&root, &["hara", "trace", "SG-CL-001", "--format", "dot"]);
    assert!(after.contains("\\nfail\\n"), "the ingested verdict is drawn: {after}");
    assert!(after.contains("fillcolor=\"#fdecea\""), "{after}");
    write(&root, ".syscribe/results.json", "{\"schema_version\":\"1\",\"format\":\"cargo-json\",\"source\":\"t\",\"ingested_at_unix\":0,\"count\":1,\"by_leaf\":{\"chain\":\"pass\"}}");
    let passing = out_in(&root, &["hara", "trace", "SG-CL-001", "--format", "dot"]);
    assert!(passing.contains("\\npass\\n"), "{passing}");
    root.parent().map(std::fs::remove_dir_all);
}

#[test]
fn connectivity_draws_the_safety_and_security_links_from_analysis_elements() {
    let sg = out(&["connectivity", "SG-ENG-001"]);
    assert!(sg.contains("[hazardousEventRef] Safety::HARA::HE-ENG-001"), "a goal reaches its hazardous events: {sg}");
    let ft = out(&["connectivity", "FT-ENG-001"]);
    assert!(ft.contains("[topEvent] Safety::HARA::SG-ENG-001"), "a fault tree reaches its goal: {ft}");
    assert!(ft.contains("[faultTreeInput]"), "gate inputs: {ft}");
    let at = out(&["connectivity", "AT-ENG-001"]);
    assert!(at.contains("[threatRef]") && at.contains("[attackTreeInput]"), "{at}");
    let csg = out(&["connectivity", "CSG-ENG-001"]);
    assert!(csg.contains("[threatScenarioRef]"), "{csg}");
    let ts = out(&["connectivity", "TS-ENG-001"]);
    assert!(ts.contains("[damageScenarioRef]"), "{ts}");
    let sc = out(&["connectivity", "SC-ENG-001"]);
    assert!(sc.contains("[implementsGoal]"), "{sc}");
    // Explicit kinds still work, and a structural root still walks structure only.
    let kinds = out(&["connectivity", "SG-ENG-001", "--kinds", "hazardousEventRef"]);
    assert!(kinds.contains("[hazardousEventRef]"));
    let arch = out(&["connectivity", "System::EngineECU", "--depth", "1"]);
    assert!(!arch.contains("hazardousEventRef"));
    assert!(!run_in(&auto(), &["connectivity", "SG-ENG-001", "--kinds", "nonsense"]).status.success());
}
