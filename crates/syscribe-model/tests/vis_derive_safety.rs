//! GH #223: the derived safety diagrams — `FaultTree`, `AttackTree` and
//! `SafetyCase` (GSN) — through the real walker and validator. The generated
//! IR for the fixture model is pinned by golden JSON snapshots under
//! `tests/vis_snapshots/derived/` — refresh with
//! `SYSCRIBE_UPDATE_SNAPSHOTS=1 cargo test -p syscribe-model --test
//! vis_derive_safety` and review the diff. The writers (Mermaid, DOT, SVG,
//! PlantUML, sprotty) and the embedded ELK layout are driven over the same
//! graphs, and the shipped example diagrams of `model_auto/` and `model_sil/`
//! are derived and validated as fixtures.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::element::RawElement;
use syscribe_model::plantuml::render_plantuml;
use syscribe_model::resolver::Resolver;
use syscribe_model::validator::{validate, Finding};
use syscribe_model::vis::sprotty::to_sgraph;
use syscribe_model::vis::{
    build_graph, layout, render_dot, render_mermaid, render_svg, DiagramGraph, DiagramKind, EdgeKind, NodeKind, Tone,
};
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-vis-safety-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// The fixture: a safety goal with a fault tree (OR over an AND of an event
/// with a failure rate and an undeveloped one, and a single-point event
/// with a probability), a threat with an attack tree whose declared
/// feasibility disagrees with the roll-up, and an argument over the goal with
/// a strategy, a context, an undeveloped claim and a requirement with a test.
fn fixture_model() -> PathBuf {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "Safety/_index.md", "---\ntype: Package\nname: Safety\n---\n");
    write(
        &root,
        "Safety/SG.md",
        "---\ntype: SafetyGoal\nid: SG-DS-001\nname: Avoid harm\nstatus: approved\nasilLevel: B\nsafeState: stopped\n---\n\nGoal.\n",
    );
    write(
        &root,
        "Safety/FT.md",
        "---\ntype: FaultTree\nid: FT-DS-001\nname: Harm tree\nstatus: approved\ntopEvent: SG-DS-001\nmissionTime: \"1000 h\"\n---\n\nTree.\n",
    );
    write(
        &root,
        "Safety/FT/Top.md",
        "---\ntype: FaultTreeGate\nid: FTG-DS-001\nname: Harm occurs\ngateType: OR\ninputs: [FTG-DS-002, FTE-DS-003]\n---\n\nTop.\n",
    );
    write(
        &root,
        "Safety/FT/Both.md",
        "---\ntype: FaultTreeGate\nid: FTG-DS-002\nname: Both channels fail\ngateType: AND\ninputs: [FTE-DS-001, FTE-DS-002]\n---\n\nBoth.\n",
    );
    write(
        &root,
        "Safety/FT/A.md",
        "---\ntype: FaultTreeEvent\nid: FTE-DS-001\nname: Channel A fails\neventKind: basic\nfailureRate: 0.0001\n---\n\nA.\n",
    );
    write(
        &root,
        "Safety/FT/B.md",
        "---\ntype: FaultTreeEvent\nid: FTE-DS-002\nname: Channel B fails\neventKind: undeveloped\nprobability: 0.01\n---\n\nB.\n",
    );
    write(
        &root,
        "Safety/FT/C.md",
        "---\ntype: FaultTreeEvent\nid: FTE-DS-003\nname: Comparator fails\neventKind: basic\nprobability: 0.000001\n---\n\nC.\n",
    );
    write(&root, "Safety/Case/_index.md", "---\ntype: Package\nname: Case\n---\n");
    write(
        &root,
        "Safety/Case/Strategy.md",
        "---\ntype: Argument\nid: ARG-DS-001\nname: Argue over independent channels\nstatus: approved\nargumentType: strategy\nsupports: SG-DS-001\nevidence: [REQ-DS-001]\n---\n\nStrategy.\n",
    );
    write(
        &root,
        "Safety/Case/Context.md",
        "---\ntype: Argument\nid: ARG-DS-002\nname: Single vehicle\nstatus: approved\nargumentType: context\nsupports: ARG-DS-001\n---\n\nContext.\n",
    );
    write(
        &root,
        "Safety/Case/Claim.md",
        "---\ntype: Argument\nid: ARG-DS-003\nname: Channels fail independently\nstatus: draft\nsupports: ARG-DS-001\n---\n\nClaim.\n",
    );
    write(
        &root,
        "Safety/REQ.md",
        "---\ntype: Requirement\nid: REQ-DS-001\nname: Comparator shall trip\nstatus: approved\nderivedFromSafetyGoal: SG-DS-001\n---\n\nShall trip.\n",
    );
    write(
        &root,
        "Safety/TC.md",
        "---\ntype: TestCase\nid: TC-DS-001\nname: Trip test\nstatus: active\ntestLevel: L3\nverifies: [REQ-DS-001]\n---\n\n```gherkin\nFeature: trip\n  Scenario: s\n    Given a\n    Then b\n```\n",
    );
    write(&root, "Security/_index.md", "---\ntype: Package\nname: Security\n---\n");
    write(
        &root,
        "Security/AT.md",
        "---\ntype: AttackTree\nid: AT-DS-001\nname: Spoof the bus\nstatus: approved\nthreatRef: TS-DS-001\n---\n\nTree.\n",
    );
    write(
        &root,
        "Security/AT/Root.md",
        "---\ntype: AttackTreeGate\nid: ATG-DS-001\nname: Either way in\ngateType: OR\ninputs: [ATG-DS-002, ATS-DS-003]\n---\n\nRoot.\n",
    );
    write(
        &root,
        "Security/AT/Chain.md",
        "---\ntype: AttackTreeGate\nid: ATG-DS-002\nname: Local path\ngateType: AND\ninputs: [ATS-DS-001, ATS-DS-002]\n---\n\nChain.\n",
    );
    write(&root, "Security/AT/S1.md", "---\ntype: AttackStep\nid: ATS-DS-001\nname: Reach the port\nattackFeasibility: high\n---\n\nS1.\n");
    write(&root, "Security/AT/S2.md", "---\ntype: AttackStep\nid: ATS-DS-002\nname: Replay a frame\nattackFeasibility: medium\n---\n\nS2.\n");
    write(&root, "Security/AT/S3.md", "---\ntype: AttackStep\nid: ATS-DS-003\nname: Remote injection\nattackFeasibility: low\n---\n\nS3.\n");
    write(
        &root,
        "Security/TARA.md",
        "---\ntype: TARASheet\nid: TARA-DS-001\nname: TARA\nstatus: approved\nthreatTable:\n  - id: TS-DS-001\n    name: Spoofed request\n    attackFeasibility: high\n---\n\nTARA.\n",
    );
    write(&root, "Diagrams/_index.md", "---\ntype: Package\nname: Diagrams\n---\n");
    root
}

fn add_diagram(root: &Path, name: &str, fm: &str) {
    write(root, &format!("Diagrams/{name}.md"), &format!("---\ntype: Diagram\nname: {name}\n{fm}---\n\n{name}.\n"));
}

struct Derived {
    graph: DiagramGraph,
    findings: Vec<Finding>,
    elements: Vec<RawElement>,
    diagram: RawElement,
}

fn derive(root: &Path, qname: &str) -> Derived {
    let elements = walk_model(root).unwrap();
    let resolver = Resolver::new(&elements);
    let d = elements.iter().find(|e| e.qualified_name == qname).expect("diagram element").clone();
    let (graph, _) = build_graph(&d, &elements, &resolver).expect("an IR");
    let findings = validate(&elements)
        .findings
        .into_iter()
        .filter(|f| f.file.ends_with(&format!("{}.md", qname.replace("::", "/"))))
        .collect();
    Derived { graph, findings, elements, diagram: d }
}

fn codes(findings: &[Finding], code: &str) -> Vec<String> {
    findings.iter().filter(|f| f.code == code).map(|f| f.message.clone()).collect()
}

fn snapshot_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/vis_snapshots/derived")
}

fn assert_snapshot(name: &str, graph: &DiagramGraph) {
    let path = snapshot_dir().join(format!("{name}.json"));
    let actual = serde_json::to_string_pretty(graph).unwrap() + "\n";
    if std::env::var_os("SYSCRIBE_UPDATE_SNAPSHOTS").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("missing snapshot {}: {e} (run with SYSCRIBE_UPDATE_SNAPSHOTS=1)", path.display()));
    assert_eq!(actual, expected, "golden IR for {name} changed; review and refresh with SYSCRIBE_UPDATE_SNAPSHOTS=1");
}

fn no_links(_: &str) -> Option<String> {
    None
}

fn kinds(g: &DiagramGraph) -> Vec<(String, NodeKind)> {
    g.nodes.iter().map(|n| (n.id.clone(), n.kind)).collect()
}

// ── FaultTree ────────────────────────────────────────────────────────────────

#[test]
fn derived_fault_tree_matches_its_golden_ir() {
    let root = fixture_model();
    add_diagram(&root, "FtD", "diagramKind: FaultTree\nsubject: Safety::FT\n");
    let d = derive(&root, "Diagrams::FtD");
    let g = &d.graph;
    assert!(codes(&d.findings, "W417").is_empty() && codes(&d.findings, "W418").is_empty(), "{:?}", d.findings);
    assert_eq!(g.kind, DiagramKind::FaultTree);
    assert!(g.derived);
    // Gate symbols by gateType, event shapes by eventKind.
    assert_eq!(
        kinds(g),
        vec![
            ("s-safety-ft-a".to_string(), NodeKind::EventBasic),
            ("s-safety-ft-b".to_string(), NodeKind::EventUndeveloped),
            ("s-safety-ft-both".to_string(), NodeKind::GateAnd),
            ("s-safety-ft-c".to_string(), NodeKind::EventBasic),
            ("s-safety-ft-top".to_string(), NodeKind::GateOr),
        ]
    );
    // Edges run from a gate to each of its inputs.
    let edges: Vec<(&str, &str)> = g.edges.iter().map(|e| (e.source.as_str(), e.target.as_str())).collect();
    assert_eq!(
        edges,
        vec![
            ("s-safety-ft-both", "s-safety-ft-a"),
            ("s-safety-ft-both", "s-safety-ft-b"),
            ("s-safety-ft-top", "s-safety-ft-both"),
            ("s-safety-ft-top", "s-safety-ft-c"),
        ]
    );
    assert!(g.edges.iter().all(|e| e.kind == EdgeKind::GateInput));
    // The overlay: single point, dual-point members, root probability.
    let mark = |id: &str| g.node(id).unwrap().mark.clone().unwrap();
    let c = mark("s-safety-ft-c");
    assert_eq!((c.status.as_deref(), c.tone, c.emphasis), (Some("single point of failure"), Tone::Bad, true));
    assert_eq!(c.value.as_deref(), Some("P 1.0e-6"));
    let a = mark("s-safety-ft-a");
    assert_eq!((a.status.as_deref(), a.tone), (Some("dual-point"), Tone::Warn));
    assert_eq!(a.value.as_deref(), Some("\u{03bb} 1.0e-4/h \u{00b7} P 9.5e-2"));
    let top = mark("s-safety-ft-top");
    assert_eq!(top.status.as_deref(), Some("top event"));
    assert!(top.badges.contains(&"1 single point".to_string()), "{top:?}");
    assert_eq!(g.layout_hints.direction, syscribe_model::vis::ir::LayoutDirection::Down);
    assert_snapshot("ft", g);
}

#[test]
fn fault_tree_of_a_safety_goal_and_filters_and_w418() {
    let root = fixture_model();
    add_diagram(&root, "ByGoal", "diagramKind: FaultTree\nsubject: SG-DS-001\n");
    let d = derive(&root, "Diagrams::ByGoal");
    assert_eq!(d.graph.nodes.len(), 5, "a safety goal draws the tree naming it as topEvent");

    add_diagram(&root, "Less", "diagramKind: FaultTree\nsubject: Safety::FT\nexclude: [FTE-DS-003, Nobody]\n");
    let d = derive(&root, "Diagrams::Less");
    assert_eq!(d.graph.nodes.len(), 4);
    assert!(d.graph.edges.iter().all(|e| e.target != "s-safety-ft-c"));
    let w417 = codes(&d.findings, "W417");
    assert_eq!(w417.len(), 1, "{:?}", d.findings);
    assert!(w417[0].contains("'Nobody'"));

    add_diagram(&root, "Bad", "diagramKind: FaultTree\nsubject: Safety::TC\n");
    let d = derive(&root, "Diagrams::Bad");
    assert!(d.graph.nodes.is_empty());
    let w418 = codes(&d.findings, "W418");
    assert_eq!(w418.len(), 1, "{:?}", d.findings);
    assert!(w418[0].contains("TestCase") && w418[0].contains("FaultTree"));
}

#[test]
fn a_cyclic_fault_tree_still_draws_without_numbers() {
    let root = fixture_model();
    // Event A feeds back into the AND gate that has it as an input.
    write(
        &root,
        "Safety/FT/A.md",
        "---\ntype: FaultTreeEvent\nid: FTE-DS-001\nname: Channel A fails\neventKind: basic\nfailureRate: 0.0001\ninputs: [FTG-DS-002]\n---\n\nA.\n",
    );
    add_diagram(&root, "Cyc", "diagramKind: FaultTree\nsubject: Safety::FT\n");
    let d = derive(&root, "Diagrams::Cyc");
    assert_eq!(d.graph.nodes.len(), 5, "the structure is drawn");
    let top = d.graph.node("s-safety-ft-top").unwrap().mark.clone().unwrap();
    assert_eq!(top.status.as_deref(), Some("analysis unavailable"));
    assert!(top.value.as_deref().unwrap().contains("cycle"), "{top:?}");
    assert!(d.graph.node("s-safety-ft-c").unwrap().mark.as_ref().is_none_or(|m| m.is_detail_only()));
    assert!(render_svg(&d.graph, &no_links).is_ok(), "ELK copes with the cycle");
}

// ── AttackTree ───────────────────────────────────────────────────────────────

#[test]
fn derived_attack_tree_matches_its_golden_ir() {
    let root = fixture_model();
    add_diagram(&root, "AtD", "diagramKind: AttackTree\nsubject: Security::AT\n");
    let d = derive(&root, "Diagrams::AtD");
    let g = &d.graph;
    assert!(codes(&d.findings, "W417").is_empty() && codes(&d.findings, "W418").is_empty(), "{:?}", d.findings);
    assert_eq!(g.kind, DiagramKind::AttackTree);
    assert_eq!(
        kinds(g),
        vec![
            ("s-security-at-chain".to_string(), NodeKind::GateAnd),
            ("s-security-at-root".to_string(), NodeKind::GateOr),
            ("s-security-at-s1".to_string(), NodeKind::Step),
            ("s-security-at-s2".to_string(), NodeKind::Step),
            ("s-security-at-s3".to_string(), NodeKind::Step),
        ]
    );
    let tone = |id: &str| g.node(id).unwrap().mark.as_ref().unwrap().tone;
    assert_eq!(tone("s-security-at-s1"), Tone::Bad);
    assert_eq!(tone("s-security-at-s2"), Tone::Warn);
    assert_eq!(tone("s-security-at-s3"), Tone::Ok);
    // OR(medium AND, low) is medium; the threat declares high: W035 badge on the root.
    let root_mark = g.node("s-security-at-root").unwrap().mark.clone().unwrap();
    assert_eq!(root_mark.status.as_deref(), Some("feasibility medium"));
    assert_eq!(root_mark.badges, vec!["W035"]);
    assert_eq!(root_mark.value.as_deref(), Some("threat TS-DS-001 declares high"));
    // The easiest path runs through the AND gate (medium beats low), both its steps included.
    let hot: Vec<(&str, &str)> = g.edges.iter().filter(|e| e.kind == EdgeKind::CriticalPath).map(|e| (e.source.as_str(), e.target.as_str())).collect();
    assert_eq!(
        hot,
        vec![
            ("s-security-at-chain", "s-security-at-s1"),
            ("s-security-at-chain", "s-security-at-s2"),
            ("s-security-at-root", "s-security-at-chain"),
        ]
    );
    assert!(!g.node("s-security-at-s3").unwrap().mark.as_ref().unwrap().emphasis);
    // The badge is the validator's own W035 finding, drawn.
    let all = validate(&d.elements).findings;
    assert!(all.iter().any(|f| f.code == "W035" && f.message.contains("computed feasibility 'medium'")), "{all:?}");
    assert_snapshot("at", g);
}

// ── SafetyCase ───────────────────────────────────────────────────────────────

#[test]
fn derived_safety_case_matches_its_golden_ir() {
    let root = fixture_model();
    add_diagram(&root, "GsnD", "diagramKind: SafetyCase\nsubject: SG-DS-001\n");
    let d = derive(&root, "Diagrams::GsnD");
    let g = &d.graph;
    assert!(codes(&d.findings, "W417").is_empty() && codes(&d.findings, "W418").is_empty(), "{:?}", d.findings);
    assert_eq!(g.kind, DiagramKind::SafetyCase);
    let kind = |id: &str| g.node(id).unwrap().kind;
    assert_eq!(kind("s-safety-sg"), NodeKind::Goal);
    assert_eq!(kind("s-safety-case-strategy"), NodeKind::Strategy);
    assert_eq!(kind("s-safety-case-context"), NodeKind::Context);
    assert_eq!(kind("s-safety-case-claim"), NodeKind::UndevelopedGoal);
    assert_eq!(kind("s-safety-req"), NodeKind::Goal, "a requirement is a sub-goal");
    assert_eq!(kind("s-safety-tc"), NodeKind::Solution, "a test case is the evidence");
    let edge = |t: &str| g.edges.iter().find(|e| e.target == t).unwrap().kind;
    assert_eq!(edge("s-safety-case-context"), EdgeKind::InContextOf);
    assert_eq!(edge("s-safety-case-claim"), EdgeKind::SupportedBy);
    // Overlays: UNDEVELOPED origin, the unknown verdict of the test, the goal verdict.
    let mark = |id: &str| g.node(id).unwrap().mark.clone().unwrap();
    assert_eq!(mark("s-safety-case-claim").status.as_deref(), Some("UNDEVELOPED"));
    assert_eq!((mark("s-safety-tc").value.as_deref(), mark("s-safety-tc").tone), (Some("unknown"), Tone::Warn));
    assert_eq!(mark("s-safety-sg").value.as_deref(), Some("verdict incomplete"));
    assert!(g.node("s-safety-case-context").unwrap().mark.as_ref().is_none_or(|m| m.is_detail_only()));
    assert_snapshot("gsn", g);
}

#[test]
fn safety_case_subject_forms_and_w418() {
    let root = fixture_model();
    add_diagram(&root, "ByPackage", "diagramKind: SafetyCase\nsubject: Safety\n");
    assert!(derive(&root, "Diagrams::ByPackage").graph.node("s-safety-sg").is_some());
    add_diagram(&root, "ByArgument", "diagramKind: SafetyCase\nsubject: ARG-DS-001\n");
    assert!(derive(&root, "Diagrams::ByArgument").graph.node("s-safety-sg").is_some(), "an argument draws the goal it belongs to");
    add_diagram(&root, "Bad", "diagramKind: SafetyCase\nsubject: Safety::TC\n");
    let d = derive(&root, "Diagrams::Bad");
    assert!(d.graph.nodes.is_empty());
    assert_eq!(codes(&d.findings, "W418").len(), 1, "{:?}", d.findings);
    add_diagram(&root, "Filtered", "diagramKind: SafetyCase\nsubject: SG-DS-001\nexclude: [ARG-DS-003, Nobody]\n");
    let d = derive(&root, "Diagrams::Filtered");
    assert!(d.graph.node("s-safety-case-claim").is_none());
    assert_eq!(codes(&d.findings, "W417").len(), 1);
}

// ── writers and layout ───────────────────────────────────────────────────────

#[test]
fn writers_accept_the_safety_graphs() {
    let root = fixture_model();
    add_diagram(&root, "FtD", "diagramKind: FaultTree\nsubject: Safety::FT\n");
    add_diagram(&root, "AtD", "diagramKind: AttackTree\nsubject: Security::AT\n");
    add_diagram(&root, "GsnD", "diagramKind: SafetyCase\nsubject: SG-DS-001\n");
    let ft = derive(&root, "Diagrams::FtD");
    let at = derive(&root, "Diagrams::AtD");
    let gsn = derive(&root, "Diagrams::GsnD");

    // Mermaid: a top-down flowchart with shapes, mark text and tone classes.
    let m = render_mermaid(&ft.graph, &no_links).unwrap();
    assert!(m.starts_with("flowchart TD\n"), "{m}");
    assert!(m.contains("s_safety_ft_top([\"OR<br/>FTG-DS-001<br/>Harm occurs<br/>top event<br/>P "), "{m}");
    assert!(m.contains("s_safety_ft_both([\"AND<br/>FTG-DS-002<br/>Both channels fail\"])") || m.contains("s_safety_ft_both([\"AND<br/>FTG-DS-002<br/>Both channels fail<br/>"), "{m}");
    assert!(m.contains("s_safety_ft_top --- s_safety_ft_c"), "{m}");
    assert!(m.contains("classDef tone_bad fill:#fdecea,stroke:#b3261e,color:#222") && m.contains("class s_safety_ft_c tone_bad"), "{m}");
    let m = render_mermaid(&at.graph, &no_links).unwrap();
    assert!(m.contains("s_security_at_chain ===") || m.contains("===") , "{m}");
    let m = render_mermaid(&gsn.graph, &no_links).unwrap();
    assert!(m.contains("-.-> s_safety_case_context"), "InContextOf is dashed: {m}");
    assert!(m.contains("[/\"ARG-DS-001<br/>Argue over independent channels"), "strategy is a parallelogram: {m}");

    // DOT.
    let dot = render_dot(&at.graph);
    assert!(dot.contains("shape=invhouse") && dot.contains("shape=house") && dot.contains("penwidth=3"), "{dot}");
    assert!(dot.contains("[W035]"), "{dot}");

    // PlantUML.
    let p = render_plantuml(&ft.diagram, &ft.elements, None).expect("PlantUML maps the FaultTree kind");
    assert!(p.starts_with("@startuml FtD\n") && p.contains("hexagon \"<b>OR</b>\\nFTG-DS-001\\nHarm occurs"), "{p}");
    assert!(p.contains("usecase \"FTE-DS-001\\nChannel A fails"), "{p}");
    assert!(render_plantuml(&gsn.diagram, &gsn.elements, None).unwrap().contains("-->"), "GSN SupportedBy");

    // SVG: ELK-laid-out, symbols and the tone fills.
    let s = render_svg(&ft.graph, &no_links).expect("SVG via the embedded ELK");
    assert!(s.contains("class=\"gate-or FaultTreeGate\"") && s.contains("A 14,14 0 1 0") && s.contains("fill=\"#fdecea\""), "{s}");
    assert!(s.contains("single point of failure") && s.contains("top event"), "{s}");
    let s = render_svg(&at.graph, &no_links).unwrap();
    assert!(s.contains("class=\"edge criticalPath\"") && s.contains("stroke=\"#b3261e\""), "{s}");
    let s = render_svg(&gsn.graph, &no_links).unwrap();
    assert!(s.contains("class=\"undeveloped-goal Argument\"") && s.contains("fill=\"#fff\""), "the undeveloped diamond: {s}");

    // sprotty: roles, the mark and the label roles reach the client.
    let j = serde_json::to_value(to_sgraph(&ft.graph)).unwrap();
    assert_eq!(j["diagramKind"], "FaultTree");
    let node = j["children"].as_array().unwrap().iter().find(|c| c["id"] == "s-safety-ft-c").unwrap();
    assert_eq!(node["kind"], "event-basic");
    assert_eq!(node["mark"]["tone"], "bad");
    assert_eq!(node["style"]["strokeWidth"], 3.0);
    let roles: Vec<&str> = node["children"].as_array().unwrap().iter().filter_map(|c| c["role"].as_str()).collect();
    assert_eq!(roles, vec!["name", "line", "status", "value", "badge"], "{node}");
    let edge = j["children"].as_array().unwrap().iter().find(|c| c["type"] == "edge").unwrap();
    assert_eq!(edge["kind"], "input");
}

#[test]
fn layout_runs_top_down_with_inputs_below_their_gate_and_symbols_sized_for_their_text() {
    let root = fixture_model();
    add_diagram(&root, "FtD", "diagramKind: FaultTree\nsubject: Safety::FT\n");
    let d = derive(&root, "Diagrams::FtD");
    let sizes = syscribe_model::vis::size::size_graph_default(&d.graph);
    let l = layout(&d.graph, &sizes).expect("ELK layout");
    let top = l.nodes["s-safety-ft-top"];
    for input in ["s-safety-ft-both", "s-safety-ft-c"] {
        assert!(l.nodes[input].y >= top.y + top.h, "{input} sits below the top gate: {:?} vs {top:?}", l.nodes[input]);
    }
    assert!(l.nodes["s-safety-ft-a"].y > l.nodes["s-safety-ft-both"].y);
    // An event is its text box with the symbol hung beneath it: the box is
    // taller than the label stack by the symbol and its stub, and the text
    // stays above the symbol.
    let a = d.graph.node("s-safety-ft-a").unwrap();
    let stack: f64 = sizes.node(&a.id).unwrap().labels.iter().map(|x| x.h + 1.0).sum();
    let extent = syscribe_model::vis::shape::glyph_extent(a.kind).unwrap();
    assert!(sizes.size_of(&a.id).h >= stack + extent, "{} vs {stack} + {extent}", sizes.size_of(&a.id).h);
    let box_ = l.nodes["s-safety-ft-a"];
    let name = l.labels["s-safety-ft-a-label"];
    assert!(name.y >= box_.y + 4.0 && name.y + name.h <= box_.bottom() - extent, "{name:?} in {box_:?}");
    // A gate input sits under its gate in a bus: one trunk out of the gate's bottom centre.
    let route = &l.edges["e-input-s-safety-ft-top-s-safety-ft-c"];
    assert_eq!((route.points[0].x, route.points[0].y), (top.cx(), top.bottom()), "{route:?}");
    // No two nodes overlap.
    let ids: Vec<&String> = l.nodes.keys().collect();
    for (i, a) in ids.iter().enumerate() {
        for b in &ids[i + 1..] {
            let (p, q) = (l.nodes[*a], l.nodes[*b]);
            let overlap = p.x < q.right() && q.x < p.right() && p.y < q.bottom() && q.y < p.bottom();
            assert!(!overlap, "{a} overlaps {b}");
        }
    }
}

// ── the shipped examples ─────────────────────────────────────────────────────

fn shipped(model: &str, qname: &str, kind: DiagramKind) -> DiagramGraph {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(model);
    let elements = walk_model(&root).unwrap();
    let resolver = Resolver::new(&elements);
    let d = elements.iter().find(|e| e.qualified_name == qname).unwrap_or_else(|| panic!("{qname} in {model}")).clone();
    let (graph, issues) = build_graph(&d, &elements, &resolver).unwrap();
    assert!(issues.is_empty(), "{qname}: {issues:?}");
    let findings: Vec<_> = validate(&elements).findings.into_iter().filter(|f| f.file.ends_with(&format!("{}.md", qname.replace("::", "/")))).collect();
    assert!(findings.is_empty(), "{qname} validates clean: {findings:?}");
    assert_eq!(graph.kind, kind);
    assert!(!graph.nodes.is_empty(), "{qname} draws something");
    graph
}

#[test]
fn the_shipped_example_diagrams_derive_and_validate_clean() {
    let ft = shipped("model_auto", "Diagrams::FaultTreeEngine", DiagramKind::FaultTree);
    assert!(ft.nodes.iter().any(|n| n.mark.as_ref().is_some_and(|m| m.status.as_deref() == Some("top event"))));
    let at = shipped("model_auto", "Diagrams::AttackTreeTorqueReplay", DiagramKind::AttackTree);
    let root = at.nodes.iter().find_map(|n| n.mark.as_ref().filter(|m| m.value.is_some())).unwrap();
    assert_eq!(root.value.as_deref(), Some("matches threat TS-ENG-001"), "the example is W035-clean");
    assert!(at.edges.iter().any(|e| e.kind == EdgeKind::CriticalPath));
    let gsn = shipped("model_auto", "Diagrams::SafetyCaseEngine", DiagramKind::SafetyCase);
    assert!(gsn.nodes.iter().any(|n| n.kind == NodeKind::UndevelopedGoal) && gsn.nodes.iter().any(|n| n.kind == NodeKind::Strategy));
    let sil = shipped("model_sil", "Diagrams::FaultTreeRouteConflict", DiagramKind::FaultTree);
    assert_eq!(sil.nodes.iter().filter(|n| n.mark.as_ref().is_some_and(|m| m.tone == Tone::Bad)).count(), 1, "the 2oo2 comparator alone is a single point");
    for g in [&ft, &at, &gsn, &sil] {
        assert!(render_svg(g, &no_links).is_ok());
    }
}

// ── writer snapshots ─────────────────────────────────────────────────────────

fn writer_snapshot(name: &str, actual: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/vis_snapshots/writers").join(name);
    if std::env::var_os("SYSCRIBE_UPDATE_SNAPSHOTS").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("missing snapshot {}: {e} (run with SYSCRIBE_UPDATE_SNAPSHOTS=1)", path.display()));
    assert_eq!(actual, expected, "golden {name} changed; review and refresh with SYSCRIBE_UPDATE_SNAPSHOTS=1");
}

/// The SVG over the font-independent approximate metrics, so the snapshot does
/// not depend on the fonts of the machine running it.
fn approx_svg(g: &DiagramGraph) -> String {
    use syscribe_model::vis::metrics::ApproxMetrics;
    syscribe_model::vis::svg::render_svg_with(g, &syscribe_model::vis::size_graph(g, &ApproxMetrics), &no_links).expect("svg")
}

#[test]
fn writer_output_for_the_safety_diagrams_matches_its_snapshots() {
    let root = fixture_model();
    add_diagram(&root, "FtD", "diagramKind: FaultTree\nsubject: Safety::FT\n");
    add_diagram(&root, "AtD", "diagramKind: AttackTree\nsubject: Security::AT\n");
    add_diagram(&root, "GsnD", "diagramKind: SafetyCase\nsubject: SG-DS-001\n");
    let ft = derive(&root, "Diagrams::FtD");
    let at = derive(&root, "Diagrams::AtD");
    let gsn = derive(&root, "Diagrams::GsnD");
    writer_snapshot("ft.mmd", &render_mermaid(&ft.graph, &no_links).unwrap());
    writer_snapshot("ft.dot", &render_dot(&ft.graph));
    writer_snapshot("ft.svg", &approx_svg(&ft.graph));
    writer_snapshot("at.mmd", &render_mermaid(&at.graph, &no_links).unwrap());
    writer_snapshot("at.svg", &approx_svg(&at.graph));
    writer_snapshot("gsn.mmd", &render_mermaid(&gsn.graph, &no_links).unwrap());
    writer_snapshot("gsn.dot", &render_dot(&gsn.graph));
    writer_snapshot("gsn.svg", &approx_svg(&gsn.graph));
    writer_snapshot("ft.puml", &render_plantuml(&ft.diagram, &ft.elements, None).unwrap());
    writer_snapshot("at.puml", &render_plantuml(&at.diagram, &at.elements, None).unwrap());
    writer_snapshot("gsn.puml", &render_plantuml(&gsn.diagram, &gsn.elements, None).unwrap());
}
