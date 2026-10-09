//! GH #223 (phase 2): the derived analysis graphs — `Traceability`
//! (hazard-to-test), `ZoneConduit` (IEC 62443) and `ThreatGraph` (TARA) —
//! through the real walker and validator. The generated IR is pinned by golden
//! JSON snapshots under `tests/vis_snapshots/derived/` (refresh with
//! `SYSCRIBE_UPDATE_SNAPSHOTS=1 cargo test -p syscribe-model --test
//! vis_derive_analysis` and review the diff); every writer and the embedded ELK
//! layout are driven over the same graphs, and the shipped example diagrams of
//! `model_auto/` and `model_sil/` are derived as fixtures.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::cyber_config::CyberConfig;
use syscribe_model::element::RawElement;
use syscribe_model::plantuml::render_plantuml;
use syscribe_model::resolver::Resolver;
use syscribe_model::safety_case::Verdict;
use syscribe_model::validator::{validate, Finding};
use syscribe_model::vis::derive::traceability::graph_with_verdicts;
use syscribe_model::vis::Issue;
use syscribe_model::vis::sprotty::to_sgraph;
use syscribe_model::vis::{
    build_graph, build_graph_with, build_subject_graph, embedded_kinds_of, layout, render_dot, render_mermaid, render_svg, DiagramGraph,
    DiagramKind, EdgeKind, NodeKind, Tone,
};
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-vis-analysis-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn pkg(root: &Path, rel: &str, name: &str) {
    write(root, &format!("{rel}/_index.md"), &format!("---\ntype: Package\nname: {name}\n---\n"));
}

/// The traceability fixture: a hazard answered by a goal with a fault tree and
/// an argument; a parent requirement with two leaves (one tested, one with
/// neither test nor satisfier) and a parent with no integration test; and a
/// second hazard no goal answers.
fn trace_model() -> PathBuf {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    pkg(&root, "Safety", "Safety");
    write(
        &root,
        "Safety/HE-A.md",
        "---\ntype: HazardousEvent\nid: HE-DT-001\nname: Loss of braking\nstatus: approved\nseverity: S3\nexposure: E4\ncontrollability: C3\n---\n\nHE.\n",
    );
    write(&root, "Safety/HE-B.md", "---\ntype: HazardousEvent\nid: HE-DT-002\nname: Unanswered hazard\nstatus: approved\n---\n\nHE.\n");
    write(
        &root,
        "Safety/SG.md",
        "---\ntype: SafetyGoal\nid: SG-DT-001\nname: Keep braking\nstatus: approved\nasilLevel: D\nsafeState: stopped\nhazardousEvents: [HE-DT-001]\n---\n\nGoal.\n",
    );
    write(&root, "Safety/FT.md", "---\ntype: FaultTree\nid: FT-DT-001\nname: Braking tree\nstatus: approved\ntopEvent: SG-DT-001\n---\n\nTree.\n");
    write(
        &root,
        "Safety/ARG.md",
        "---\ntype: Argument\nid: ARG-DT-001\nname: Argue over channels\nstatus: approved\nargumentType: strategy\nsupports: SG-DT-001\n---\n\nArg.\n",
    );
    write(&root, "ADRs/ADR-DT-001.md", "---\ntype: ADR\nid: ADR-DT-001\nname: Split braking\nstatus: accepted\n---\n\nSplit.\n");
    write(
        &root,
        "Req/REQ-DT-001.md",
        "---\ntype: Requirement\nid: REQ-DT-001\nname: Brake on demand\nstatus: approved\nasilLevel: D\nreqDomain: system\nderivedFromSafetyGoal: SG-DT-001\n---\n\nShall brake.\n",
    );
    write(
        &root,
        "Req/REQ-DT-002.md",
        "---\ntype: Requirement\nid: REQ-DT-002\nname: Detect pedal\nstatus: approved\nreqDomain: system\nderivedFrom: [REQ-DT-001]\nbreakdownAdr: ADR-DT-001\n---\n\nShall detect.\n",
    );
    write(
        &root,
        "Req/REQ-DT-003.md",
        "---\ntype: Requirement\nid: REQ-DT-003\nname: Apply pressure\nstatus: approved\nreqDomain: system\nderivedFrom: [REQ-DT-001]\nbreakdownAdr: ADR-DT-001\n---\n\nShall apply.\n",
    );
    write(&root, "Arch/Pedal.md", "---\ntype: PartDef\nname: Pedal\ndomain: system\nsatisfies: [REQ-DT-002]\n---\n\nPedal.\n");
    write(
        &root,
        "Verification/TC-DT-001.md",
        "---\ntype: TestCase\nid: TC-DT-001\nname: Pedal test\nstatus: active\ntestLevel: L2\nverifies: [REQ-DT-002]\n---\n\n```gherkin\nFeature: pedal\n  Scenario: s\n    Given a\n    Then b\n```\n",
    );
    root
}

fn model_of(root: &Path) -> Vec<RawElement> {
    walk_model(root).unwrap()
}

fn find<'a>(els: &'a [RawElement], q: &str) -> &'a RawElement {
    els.iter().find(|e| e.qualified_name == q).unwrap_or_else(|| panic!("no element {q}"))
}

fn subject_graph(els: &[RawElement], q: &str, kind: DiagramKind) -> (DiagramGraph, Vec<Issue>) {
    let r = Resolver::new(els);
    build_subject_graph(find(els, q), kind, els, &r)
}

fn add_diagram(root: &Path, name: &str, fm: &str) {
    write(root, &format!("Diagrams/{name}.md"), &format!("---\ntype: Diagram\nname: {name}\n{fm}---\n\n{name}.\n"));
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

fn writer_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/vis_snapshots/writers")
}

fn writer_snapshot(name: &str, actual: &str) {
    let path = writer_dir().join(name);
    if std::env::var_os("SYSCRIBE_UPDATE_SNAPSHOTS").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("missing snapshot {}: {e} (run with SYSCRIBE_UPDATE_SNAPSHOTS=1)", path.display()));
    assert_eq!(actual, expected, "writer output {name} changed; review and refresh with SYSCRIBE_UPDATE_SNAPSHOTS=1");
}

fn no_links(_: &str) -> Option<String> {
    None
}

fn mark(g: &DiagramGraph, id: &str) -> syscribe_model::vis::NodeMark {
    g.node(id).unwrap_or_else(|| panic!("no node {id}")).mark.clone().unwrap_or_else(|| panic!("no mark on {id}"))
}

fn codes(findings: &[Finding], code: &str) -> usize {
    findings.iter().filter(|f| f.code == code).count()
}

// ── Traceability ─────────────────────────────────────────────────────────────

#[test]
fn traceability_draws_the_hazard_to_test_spine_with_its_gaps() {
    let root = trace_model();
    let els = model_of(&root);
    let (g, issues) = subject_graph(&els, "Safety::SG", DiagramKind::Traceability);
    assert!(issues.is_empty(), "{issues:?}");
    assert_eq!(g.kind, DiagramKind::Traceability);
    assert!(g.derived);

    let kinds: Vec<(&str, NodeKind)> = g.nodes.iter().map(|n| (n.id.as_str(), n.kind)).collect();
    assert_eq!(
        kinds,
        vec![
            ("s-safety-sg", NodeKind::Goal),
            ("s-safety-he-a", NodeKind::Block),
            ("s-safety-ft", NodeKind::Block),
            ("s-safety-arg", NodeKind::Goal),
            ("s-req-req-dt-001", NodeKind::Requirement),
            ("s-req-req-dt-002", NodeKind::Requirement),
            ("s-verification-tc-dt-001", NodeKind::TestCase),
            ("s-req-req-dt-003", NodeKind::Requirement),
        ]
    );
    // Links point upstream, as held (OSLC): goal -> hazard, tree/argument/requirement -> goal,
    // child -> parent, test -> requirement.
    let edge = |k: EdgeKind, s: &str, t: &str| g.edges.iter().any(|e| e.kind == k && e.source == s && e.target == t);
    assert!(edge(EdgeKind::Trace, "s-safety-sg", "s-safety-he-a"));
    assert!(edge(EdgeKind::Trace, "s-safety-ft", "s-safety-sg"));
    assert!(edge(EdgeKind::Trace, "s-safety-arg", "s-safety-sg"));
    assert!(edge(EdgeKind::Derive, "s-req-req-dt-001", "s-safety-sg"));
    assert!(edge(EdgeKind::Derive, "s-req-req-dt-002", "s-req-req-dt-001"));
    assert!(edge(EdgeKind::Verify, "s-verification-tc-dt-001", "s-req-req-dt-002"));
    assert_eq!(g.edges.len(), 7, "{:?}", g.edges);

    // Gaps are the validator's own findings, as red badges.
    let all = validate(&els).findings;
    let leaf_untested = mark(&g, "s-req-req-dt-003");
    assert_eq!(leaf_untested.tone, Tone::Bad);
    assert!(leaf_untested.badges.contains(&"W002 no test".to_string()), "{leaf_untested:?}");
    assert!(leaf_untested.badges.contains(&"W300 unsatisfied".to_string()), "{leaf_untested:?}");
    assert!(all.iter().any(|f| f.code == "W002" && f.message.contains("REQ-DT-003")), "{all:?}");
    assert!(all.iter().any(|f| f.code == "W300" && f.message.contains("REQ-DT-003")), "{all:?}");
    let parent = mark(&g, "s-req-req-dt-001");
    assert_eq!(parent.tone, Tone::Bad);
    assert_eq!(parent.badges, vec!["W305 no integration test"]);
    assert!(all.iter().any(|f| f.code == "W305" && f.message.contains("REQ-DT-001")), "{all:?}");
    assert_eq!(parent.value.as_deref(), Some("ASIL D"));
    // The tested, satisfied leaf has no gap; a derived diagram has no results, so its test is unknown.
    let tested = mark(&g, "s-req-req-dt-002");
    assert!(tested.badges.is_empty(), "{tested:?}");
    assert_eq!(tested.tone, Tone::Warn);
    let tc = mark(&g, "s-verification-tc-dt-001");
    assert_eq!((tc.status.as_deref(), tc.value.as_deref(), tc.tone), (Some("unknown"), Some("L2 · active"), Tone::Warn));
    // The goal and the hazard take the worst of what is below them.
    assert_eq!(mark(&g, "s-safety-sg").tone, Tone::Bad);
    assert_eq!(mark(&g, "s-safety-sg").value.as_deref(), Some("ASIL D"));
    let he = mark(&g, "s-safety-he-a");
    assert_eq!((he.value.as_deref(), he.tone), (Some("ASIL D"), Tone::Bad), "ASIL determined from S3 E4 C3");
    assert_snapshot("trace", &g);
}

#[test]
fn traceability_shows_real_verdicts_and_clears_the_gaps_when_tests_exist() {
    let root = trace_model();
    // Cover the two gaps: a test for the untested leaf, an integration test for the parent.
    write(
        &root,
        "Verification/TC-DT-002.md",
        "---\ntype: TestCase\nid: TC-DT-002\nname: Pressure test\nstatus: active\ntestLevel: L2\nverifies: [REQ-DT-003]\n---\n\nT.\n",
    );
    write(
        &root,
        "Verification/TC-DT-003.md",
        "---\ntype: TestCase\nid: TC-DT-003\nname: Vehicle test\nstatus: active\ntestLevel: L5\nverifies: [REQ-DT-001]\n---\n\nT.\n",
    );
    write(&root, "Arch/Brake.md", "---\ntype: PartDef\nname: Brake\ndomain: system\nsatisfies: [REQ-DT-003]\n---\n\nBrake.\n");
    let els = model_of(&root);
    let r = Resolver::new(&els);
    let sg = find(&els, "Safety::SG");
    let all_pass = |_: &RawElement| Verdict::Pass;
    let (g, _) = graph_with_verdicts(sg, &els, &r, &all_pass);
    for id in ["s-req-req-dt-001", "s-req-req-dt-002", "s-req-req-dt-003"] {
        let m = mark(&g, id);
        assert!(m.badges.is_empty(), "{id}: {m:?}");
        assert_eq!(m.tone, Tone::Ok, "{id}");
    }
    assert_eq!(mark(&g, "s-safety-sg").tone, Tone::Ok);
    assert_eq!(mark(&g, "s-verification-tc-dt-002").status.as_deref(), Some("pass"));

    let one_fails = |tc: &RawElement| if tc.frontmatter.id.as_deref() == Some("TC-DT-002") { Verdict::Fail } else { Verdict::Pass };
    let (g, _) = graph_with_verdicts(sg, &els, &r, &one_fails);
    assert_eq!(mark(&g, "s-verification-tc-dt-002").tone, Tone::Bad);
    assert_eq!(mark(&g, "s-req-req-dt-003").tone, Tone::Bad);
    assert_eq!(mark(&g, "s-req-req-dt-001").tone, Tone::Bad, "a failing leaf fails its parent");
    assert_eq!(mark(&g, "s-req-req-dt-002").tone, Tone::Ok, "its sibling is untouched");
    assert_eq!(mark(&g, "s-safety-sg").tone, Tone::Bad);
}

#[test]
fn traceability_subjects_hazard_requirement_package_and_wrong_types() {
    let root = trace_model();
    let els = model_of(&root);

    // A hazardous event: the goals that answer it, and what hangs off them.
    let (g, issues) = subject_graph(&els, "Safety::HE-A", DiagramKind::Traceability);
    assert!(issues.is_empty());
    assert!(g.node("s-safety-sg").is_some() && g.node("s-safety-he-a").is_some());

    // An unanswered hazard stands alone, flagged.
    let (g, _) = subject_graph(&els, "Safety::HE-B", DiagramKind::Traceability);
    assert_eq!(g.nodes.len(), 1);
    let m = mark(&g, "s-safety-he-b");
    assert_eq!((m.tone, m.badges), (Tone::Bad, vec!["no goal".to_string()]));

    // A requirement: its lineage only (itself, ancestors, descendants, the goal it traces to).
    let (g, _) = subject_graph(&els, "Req::REQ-DT-002", DiagramKind::Traceability);
    let ids: Vec<&str> = g.nodes.iter().map(|n| n.id.as_str()).collect();
    assert!(ids.contains(&"s-req-req-dt-002") && ids.contains(&"s-req-req-dt-001") && ids.contains(&"s-safety-sg"), "{ids:?}");
    assert!(!ids.contains(&"s-req-req-dt-003"), "a sibling is not lineage: {ids:?}");

    // A package: every goal and every unanswered hazard under it.
    let (g, _) = subject_graph(&els, "Safety", DiagramKind::Traceability);
    assert!(g.node("s-safety-sg").is_some() && g.node("s-safety-he-b").is_some());

    // The wrong type is W418 and an empty graph.
    let (g, issues) = subject_graph(&els, "Verification::TC-DT-001", DiagramKind::Traceability);
    assert!(g.nodes.is_empty());
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "W418");
    assert!(issues[0].message.contains("TestCase") && issues[0].message.contains("Traceability"));

    // A goal with nothing under it is a gap.
    write(&root, "Safety/SG2.md", "---\ntype: SafetyGoal\nid: SG-DT-002\nname: Bare goal\nstatus: approved\nasilLevel: B\nsafeState: s\n---\n\nG.\n");
    let els = model_of(&root);
    let (g, _) = subject_graph(&els, "Safety::SG2", DiagramKind::Traceability);
    let m = mark(&g, "s-safety-sg2");
    assert_eq!((m.tone, m.badges), (Tone::Bad, vec!["no requirement".to_string()]));
}

#[test]
fn traceability_diagram_element_filters_and_snapshots() {
    let root = trace_model();
    add_diagram(&root, "TraceD", "diagramKind: Traceability\nsubject: Safety::SG\nexclude: [FT-DT-001, Nobody]\n");
    let els = model_of(&root);
    let r = Resolver::new(&els);
    let d = find(&els, "Diagrams::TraceD");
    let (g, issues) = build_graph(d, &els, &r).expect("an IR");
    assert!(g.node("s-safety-ft").is_none(), "excluded by id");
    assert!(g.node("s-safety-arg").is_some());
    assert_eq!(issues.iter().filter(|i| i.code == "W417").count(), 1, "{issues:?}");

    // The alias spelling parses to the same kind.
    assert_eq!(DiagramKind::parse(Some("HazardTrace")), Some(DiagramKind::Traceability));
    // The element is valid: no W418 / W417 from the validator beyond the deliberate Nobody.
    let all = validate(&els).findings;
    assert!(all.iter().filter(|f| f.code == "W418").count() == 0);

    let (g, _) = subject_graph(&els, "Safety::SG", DiagramKind::Traceability);
    // Every writer and the layout accept the graph.
    let svg = render_svg(&g, &no_links).expect("svg");
    assert!(svg.contains("W305 no integration test"), "badge in the SVG");
    let mmd = render_mermaid(&g, &no_links).expect("mermaid");
    assert!(mmd.starts_with("flowchart LR"), "{mmd}");
    assert!(mmd.contains("classDef tone_bad"), "{mmd}");
    let dot = render_dot(&g);
    assert!(dot.contains("rankdir=LR"), "left to right: {dot}");
    let sg = to_sgraph(&g);
    assert!(serde_json::to_string(&sg).unwrap().contains("\"mark\""));
    let l = layout(&g, &syscribe_model::vis::size::size_graph_default(&g)).expect("layout");
    assert_eq!(l.nodes.len(), g.nodes.len());
    // Hazards read first, left to right: the goal sits right of its hazard, the tests rightmost.
    assert!(l.nodes["s-safety-sg"].x > l.nodes["s-safety-he-a"].x);
    assert!(l.nodes["s-verification-tc-dt-001"].x > l.nodes["s-req-req-dt-002"].x);
    assert!(l.nodes["s-req-req-dt-002"].x > l.nodes["s-safety-sg"].x);
    writer_snapshot("trace.mmd", &mmd);
    writer_snapshot("trace.dot", &dot);
    let puml = render_plantuml(d, &els, None).expect("PlantUML maps the kind");
    assert!(puml.contains("W305 no integration test") && puml.contains("..>"), "{puml}");
}

// ── ZoneConduit ──────────────────────────────────────────────────────────────

/// Two zones (one with an SL gap) and a weak conduit, a second conduit that
/// holds, a control allocated to a member, and a part that joins a zone by `inZone:`.
fn zone_model() -> PathBuf {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    pkg(&root, "Arch", "Arch");
    write(&root, "Arch/Fc.md", "---\ntype: PartDef\nname: FlightController\ndomain: software\n---\n\nFc.\n");
    write(&root, "Arch/Gw.md", "---\ntype: PartDef\nname: Gateway\ndomain: software\ninZone: ZN-ZC-002\n---\n\nGw.\n");
    write(&root, "Arch/Tel.md", "---\ntype: PartDef\nname: Telemetry\ndomain: software\n---\n\nTel.\n");
    pkg(&root, "Sec", "Sec");
    write(
        &root,
        "Sec/ZN-ZC-001.md",
        "---\ntype: Zone\nid: ZN-ZC-001\nname: Control\nstatus: approved\ntargetSL: 3\nachievedSL: 3\nmembers: [Arch::Fc]\n---\n\nZ.\n",
    );
    write(&root, "Sec/ZN-ZC-002.md", "---\ntype: Zone\nid: ZN-ZC-002\nname: Link\nstatus: approved\ntargetSL: 2\nachievedSL: 1\n---\n\nZ.\n");
    write(
        &root,
        "Sec/ZN-ZC-003.md",
        "---\ntype: Zone\nid: ZN-ZC-003\nname: Ground\nstatus: draft\ntargetSL: 2\nmembers: [Arch::Tel, Arch::Missing]\n---\n\nZ.\n",
    );
    write(
        &root,
        "Sec/CD-ZC-001.md",
        "---\ntype: Conduit\nid: CD-ZC-001\nname: Link to control\nstatus: approved\nfromZone: ZN-ZC-002\ntoZone: ZN-ZC-001\nachievedSL: 2\nprotocols: [MAVLink]\n---\n\nC.\n",
    );
    write(
        &root,
        "Sec/CD-ZC-002.md",
        "---\ntype: Conduit\nid: CD-ZC-002\nname: Ground to link\nstatus: approved\nfromZone: ZN-ZC-003\ntoZone: ZN-ZC-002\nachievedSL: 2\n---\n\nC.\n",
    );
    write(
        &root,
        "Sec/SC-ZC-001.md",
        "---\ntype: SecurityControl\nid: SC-ZC-001\nname: Authenticate commands\nstatus: approved\ncontrolType: prevention\n---\n\nSC.\n",
    );
    write(&root, "Sec/Alloc.md", "---\ntype: Allocation\nname: Auth to Fc\nallocatedFrom: SC-ZC-001\nallocatedTo: Arch::Fc\n---\n\nA.\n");
    root
}

#[test]
fn zone_conduit_draws_compound_zones_and_weak_conduits() {
    let root = zone_model();
    let els = model_of(&root);
    let (g, issues) = subject_graph(&els, "Sec", DiagramKind::ZoneConduit);
    assert!(issues.is_empty(), "{issues:?}");
    assert_eq!(g.kind, DiagramKind::ZoneConduit);
    assert!(g.layout_hints.hierarchical);

    // Zones are compound nodes holding their parts (members: and inZone:) and controls.
    let zone = |q: &str| g.node(q).unwrap_or_else(|| panic!("no zone {q}"));
    assert_eq!(zone("s-sec-zn-zc-001").kind, NodeKind::Zone);
    let children = |zid: &str| -> Vec<String> { g.children_of(zid).map(|n| n.label.clone()).collect() };
    assert_eq!(children("s-sec-zn-zc-001"), vec!["FlightController", "SC-ZC-001"]);
    assert_eq!(children("s-sec-zn-zc-002"), vec!["Gateway"], "inZone: places a part");
    assert_eq!(children("s-sec-zn-zc-003"), vec!["Telemetry", "Arch::Missing"], "an unresolved member is still drawn, dashed");
    let missing = g.children_of("s-sec-zn-zc-003").find(|n| n.label == "Arch::Missing").unwrap();
    assert!(!missing.resolved);

    // SL labels and tones: red with a badge on a gap, green when met, amber when unknown.
    let gap = mark(&g, "s-sec-zn-zc-002");
    assert_eq!((gap.tone, gap.status.as_deref()), (Tone::Bad, Some("SL target 2 / achieved 1")));
    assert!(gap.badges.contains(&"SL gap".to_string()) && gap.badges.contains(&"no controls".to_string()), "{gap:?}");
    assert_eq!(mark(&g, "s-sec-zn-zc-001").tone, Tone::Ok);
    assert_eq!(mark(&g, "s-sec-zn-zc-001").badges, Vec::<String>::new());
    let unknown = mark(&g, "s-sec-zn-zc-003");
    assert_eq!((unknown.tone, unknown.status.as_deref()), (Tone::Warn, Some("SL target 2 / achieved ?")));

    // Conduits: weak where achieved < the highest target of the joined zones.
    let weak = g.edges.iter().find(|e| e.element_ref.as_deref() == Some("Sec::CD-ZC-001")).unwrap();
    assert_eq!(weak.kind, EdgeKind::WeakConduit);
    assert_eq!(weak.label.as_deref(), Some("CD-ZC-001 \u{b7} SL 2/3 weak \u{b7} MAVLink"));
    assert_eq!((weak.source.as_str(), weak.target.as_str()), ("s-sec-zn-zc-002", "s-sec-zn-zc-001"));
    let ok = g.edges.iter().find(|e| e.element_ref.as_deref() == Some("Sec::CD-ZC-002")).unwrap();
    assert_eq!(ok.kind, EdgeKind::Conduit);
    assert_eq!(ok.label.as_deref(), Some("CD-ZC-002 \u{b7} SL 2/2"));
    // The picture and the validator agree: W950 on the zone, W951 on the weak conduit.
    let all = validate(&els).findings;
    assert_eq!(codes(&all, "W950"), 1, "{all:?}");
    assert_eq!(codes(&all, "W951"), 1, "{all:?}");
    assert_snapshot("zones", &g);

    // Writers and layout.
    let svg = render_svg(&g, &no_links).expect("svg");
    assert!(svg.contains("SL gap") && svg.contains("#b3261e"), "red weak conduit and gap");
    let mmd = render_mermaid(&g, &no_links).unwrap();
    assert!(mmd.contains("subgraph s_sec_zn_zc_001") && mmd.contains("==="), "zones are subgraphs, a weak conduit a thick link: {mmd}");
    let dot = render_dot(&g);
    assert!(dot.contains("subgraph cluster_s_sec_zn_zc_001") && dot.contains("compound=true") && dot.contains("lhead="), "{dot}");
    assert!(dot.contains("penwidth=3"), "weak conduit is heavy: {dot}");
    writer_snapshot("zones.mmd", &mmd);
    writer_snapshot("zones.dot", &dot);
    let l = layout(&g, &syscribe_model::vis::size::size_graph_default(&g)).expect("ELK lays out compound zones");
    let z = l.nodes["s-sec-zn-zc-001"];
    let member = l.nodes[g.children_of("s-sec-zn-zc-001").next().unwrap().id.as_str()];
    assert!(member.x >= z.x && member.y >= z.y && member.x + member.w <= z.x + z.w, "a member sits inside its zone: {member:?} in {z:?}");
}

#[test]
fn zone_conduit_subjects_zone_conduit_and_wrong_types() {
    let root = zone_model();
    let els = model_of(&root);
    // A zone: itself, its conduits and the zones at their far ends.
    let (g, _) = subject_graph(&els, "Sec::ZN-ZC-001", DiagramKind::ZoneConduit);
    let zones: Vec<&str> = g.nodes.iter().filter(|n| n.kind == NodeKind::Zone).map(|n| n.id.as_str()).collect();
    assert_eq!(zones, vec!["s-sec-zn-zc-001", "s-sec-zn-zc-002"]);
    assert_eq!(g.edges.len(), 1);
    // A conduit: it and its two zones.
    let (g, _) = subject_graph(&els, "Sec::CD-ZC-002", DiagramKind::ZoneConduit);
    assert_eq!(g.nodes.iter().filter(|n| n.kind == NodeKind::Zone).count(), 2);
    assert_eq!(g.edges.len(), 1);
    // A part is the wrong type.
    let (g, issues) = subject_graph(&els, "Arch::Fc", DiagramKind::ZoneConduit);
    assert!(g.nodes.is_empty());
    assert_eq!(issues[0].code, "W418");
    // A package with no zone is an empty diagram, also W418.
    let (g, issues) = subject_graph(&els, "Arch", DiagramKind::ZoneConduit);
    assert!(g.nodes.is_empty() && issues[0].code == "W418", "{issues:?}");
}

#[test]
fn zone_logic_is_shared_and_controls_come_from_allocations() {
    use syscribe_model::zones::{conduit_required_sl, conduit_status, controls_by_zone, zone_gap, zone_members, ConduitStatus};
    let root = zone_model();
    let els = model_of(&root);
    let r = Resolver::new(&els);
    let by = controls_by_zone(&els, &r);
    assert_eq!(by["Sec::ZN-ZC-001"].iter().cloned().collect::<Vec<_>>(), vec!["SC-ZC-001".to_string()]);
    assert!(by["Sec::ZN-ZC-002"].is_empty());
    assert_eq!(zone_members(find(&els, "Sec::ZN-ZC-002"), &els, &r), vec!["Arch::Gw".to_string()]);
    assert_eq!(zone_gap(find(&els, "Sec::ZN-ZC-002")), Some(true));
    assert_eq!(zone_gap(find(&els, "Sec::ZN-ZC-003")), None);
    let c = find(&els, "Sec::CD-ZC-001");
    let req = conduit_required_sl(c, &els, &r);
    assert_eq!((req, conduit_status(c, req)), (Some(3), ConduitStatus::Weak));
    let c2 = find(&els, "Sec::CD-ZC-002");
    assert_eq!(conduit_status(c2, conduit_required_sl(c2, &els, &r)), ConduitStatus::Ok);
}

// ── ThreatGraph ──────────────────────────────────────────────────────────────

fn tara_model(toml: Option<&str>) -> PathBuf {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    if let Some(t) = toml {
        write(&root, ".syscribe.toml", t);
    }
    pkg(&root, "Sec", "Sec");
    write(&root, "Sec/ASSET-TG-001.md", "---\ntype: Asset\nid: ASSET-TG-001\nname: Command link\nstatus: approved\ncybersecurityProperties: [integrity]\n---\n\nA.\n");
    write(
        &root,
        "Sec/TARA.md",
        "---\ntype: TARASheet\nid: TARA-TG-001\nname: TARA\nstatus: approved\n\
damageTable:\n  - id: DS-TG-001\n    name: Hijack\n    damageSeverity: negligible\n    assets: [ASSET-TG-001]\n  - id: DS-TG-002\n    name: Noise\n    damageSeverity: severe\n\
threatTable:\n  - id: TS-TG-001\n    name: Spoof a command\n    attackFeasibility: high\n    damageScenarios: [DS-TG-001]\n  - id: TS-TG-002\n    name: Jam the link\n    attackFeasibility: low\n    damageScenarios: [DS-TG-002]\n  - id: TS-TG-003\n    name: Untreated\n    attackFeasibility: low\n    damageScenarios: [DS-TG-002]\n\
goalTable:\n  - id: CSG-TG-001\n    name: Authenticate commands\n    calLevel: CAL2\n    securityProperty: integrity\n    threatScenarios: [TS-TG-001]\n  - id: CSG-TG-002\n    name: Keep the link up\n    calLevel: CAL2\n    securityProperty: availability\n    threatScenarios: [TS-TG-002]\n\
controlTable:\n  - id: SC-TG-001\n    name: Sign commands\n    controlType: prevention\n    implementsGoals: [CSG-TG-001]\n---\n\nTARA.\n",
    );
    root
}

#[test]
fn threat_graph_draws_the_tara_chain_with_gaps() {
    let root = tara_model(None);
    let els = model_of(&root);
    let (g, issues) = subject_graph(&els, "Sec::TARA", DiagramKind::ThreatGraph);
    assert!(issues.is_empty(), "{issues:?}");
    assert_eq!(g.kind, DiagramKind::ThreatGraph);
    let edge = |k: EdgeKind, s: &str, t: &str| g.edges.iter().any(|e| e.kind == k && e.source == s && e.target == t);
    // Threat -> damage -> asset -> goal -> control.
    assert!(edge(EdgeKind::Impacts, "s-sec-tara-ts-tg-001", "s-sec-tara-ds-tg-001"));
    assert!(edge(EdgeKind::Affects, "s-sec-tara-ds-tg-001", "s-sec-asset-tg-001"));
    assert!(edge(EdgeKind::ProtectedBy, "s-sec-asset-tg-001", "s-sec-tara-csg-tg-001"));
    assert!(edge(EdgeKind::ImplementedBy, "s-sec-tara-csg-tg-001", "s-sec-tara-sc-tg-001"));
    // A damage scenario with no asset links straight to the goal.
    assert!(edge(EdgeKind::ProtectedBy, "s-sec-tara-ds-tg-002", "s-sec-tara-csg-tg-002"));
    // Gaps: a goal no control implements (W802), a threat no goal treats.
    let goal2 = mark(&g, "s-sec-tara-csg-tg-002");
    assert_eq!((goal2.tone, goal2.badges), (Tone::Bad, vec!["W802 no control".to_string()]));
    assert_eq!(mark(&g, "s-sec-tara-csg-tg-001").tone, Tone::Ok);
    let untreated = mark(&g, "s-sec-tara-ts-tg-003");
    assert_eq!(untreated.badges, vec!["no goal"]);
    let all = validate(&els).findings;
    assert!(all.iter().any(|f| f.code == "W802" && f.message.contains("CSG-TG-002")), "{all:?}");
    assert_snapshot("threat", &g);

    {
        let kind = DiagramKind::ThreatGraph;
        let svg = render_svg(&g, &no_links).expect("svg");
        assert!(svg.contains("W802 no control") && svg.contains("no goal"));
        let mmd = render_mermaid(&g, &no_links).unwrap();
        assert!(mmd.starts_with("flowchart LR"), "{kind:?}: {mmd}");
        writer_snapshot("threat.mmd", &mmd);
        writer_snapshot("threat.dot", &render_dot(&g));
    }
}

#[test]
fn threat_graph_risk_follows_the_configured_method() {
    let default_root = tara_model(None);
    let els = model_of(&default_root);
    let r = Resolver::new(&els);
    let sheet = find(&els, "Sec::TARA");
    // negligible impact + high feasibility = medium under the default (rank sum 3).
    let (g, _) = build_subject_graph(sheet, DiagramKind::ThreatGraph, &els, &r);
    let m = mark(&g, "s-sec-tara-ts-tg-001");
    assert_eq!((m.status.as_deref(), m.tone), (Some("risk medium"), Tone::Warn));
    assert_eq!(m.value.as_deref(), Some("feasibility high"));
    // A project that overrides that cell to critical sees a critical, heavy node.
    let cfg = CyberConfig::from_toml_str("[cyber.risk_matrix.negligible]\nhigh = \"critical\"\n");
    let (g, _) = syscribe_model::vis::build_subject_graph_with(sheet, DiagramKind::ThreatGraph, &els, &r, &cfg);
    let m = mark(&g, "s-sec-tara-ts-tg-001");
    assert_eq!((m.status.as_deref(), m.tone, m.emphasis), (Some("risk critical"), Tone::Bad, true));

    // The same configuration reaches a Diagram element through build_graph_with.
    add_diagram(&default_root, "TgD", "diagramKind: ThreatGraph\nsubject: Sec::TARA\n");
    let els = model_of(&default_root);
    let r = Resolver::new(&els);
    let d = find(&els, "Diagrams::TgD");
    let (plain, _) = build_graph(d, &els, &r).unwrap();
    let (configured, _) = build_graph_with(d, &els, &r, &cfg).unwrap();
    assert_eq!(mark(&plain, "s-sec-tara-ts-tg-001").tone, Tone::Warn);
    assert_eq!(mark(&configured, "s-sec-tara-ts-tg-001").tone, Tone::Bad);
}

#[test]
fn threat_graph_subjects_and_wrong_types() {
    let root = tara_model(None);
    let els = model_of(&root);
    // A goal: the chains through it and its controls only.
    let (g, _) = subject_graph(&els, "Sec::TARA::CSG-TG-001", DiagramKind::ThreatGraph);
    assert!(g.node("s-sec-tara-ts-tg-001").is_some() && g.node("s-sec-tara-sc-tg-001").is_some());
    assert!(g.node("s-sec-tara-ts-tg-002").is_none());
    // A control reaches the chains of the goals it implements.
    let (g, _) = subject_graph(&els, "Sec::TARA::SC-TG-001", DiagramKind::ThreatGraph);
    assert!(g.node("s-sec-asset-tg-001").is_some());
    // An asset stands for the chains that harm it.
    let (g, _) = subject_graph(&els, "Sec::ASSET-TG-001", DiagramKind::ThreatGraph);
    assert!(g.node("s-sec-tara-ts-tg-001").is_some());
    // Wrong type.
    let (g, issues) = subject_graph(&els, "Sec::ASSET-TG-001", DiagramKind::Traceability);
    assert!(g.nodes.is_empty() && issues[0].code == "W418");
    write(&root, "Arch/P.md", "---\ntype: PartDef\nname: P\n---\n\nP.\n");
    let els = model_of(&root);
    let (g, issues) = subject_graph(&els, "Arch::P", DiagramKind::ThreatGraph);
    assert!(g.nodes.is_empty() && issues[0].code == "W418" && issues[0].message.contains("PartDef"), "{issues:?}");
}

// ── Detail-panel embedding ───────────────────────────────────────────────────

#[test]
fn embedded_kinds_offer_the_analysis_graphs_for_the_right_elements() {
    let els = model_of(&trace_model());
    let r = Resolver::new(&els);
    let kinds = |q: &str| embedded_kinds_of(find(&els, q), &els, &r);
    assert_eq!(kinds("Safety::HE-A"), vec![DiagramKind::Traceability]);
    // The safety kinds keep their order; the analysis graph follows.
    assert_eq!(kinds("Safety::SG"), vec![DiagramKind::SafetyCase, DiagramKind::FaultTree, DiagramKind::Traceability]);
    assert_eq!(kinds("Req::REQ-DT-002"), vec![DiagramKind::Traceability], "a requirement that traces to a goal");
    assert!(kinds("Verification::TC-DT-001").is_empty());
    let els = model_of(&zone_model());
    let r = Resolver::new(&els);
    assert_eq!(embedded_kinds_of(find(&els, "Sec::ZN-ZC-001"), &els, &r), vec![DiagramKind::ZoneConduit]);
    assert_eq!(embedded_kinds_of(find(&els, "Sec::CD-ZC-001"), &els, &r), vec![DiagramKind::ZoneConduit]);
    let els = model_of(&tara_model(None));
    let r = Resolver::new(&els);
    assert_eq!(embedded_kinds_of(find(&els, "Sec::TARA::TS-TG-001"), &els, &r), vec![DiagramKind::ThreatGraph]);
    assert_eq!(embedded_kinds_of(find(&els, "Sec::ASSET-TG-001"), &els, &r), vec![DiagramKind::ThreatGraph]);
}

#[test]
fn a_requirement_with_no_goal_has_no_traceability_panel() {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "R.md", "---\ntype: Requirement\nid: REQ-LONE-001\nname: Lone\nstatus: draft\nreqDomain: system\n---\n\nShall.\n");
    let els = model_of(&root);
    let r = Resolver::new(&els);
    assert!(embedded_kinds_of(find(&els, "R"), &els, &r).is_empty());
}

// ── The shipped examples ─────────────────────────────────────────────────────

fn shipped(model: &str) -> (Vec<RawElement>, PathBuf) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(model);
    (walk_model(&root).unwrap(), root)
}

#[test]
fn shipped_example_diagrams_derive_and_validate() {
    for (model, name, kind, min_nodes) in [
        ("model_auto", "Diagrams::TraceabilityEngine", DiagramKind::Traceability, 6),
        ("model_auto", "Diagrams::ZoneConduitEngine", DiagramKind::ZoneConduit, 10),
        ("model_auto", "Diagrams::ThreatGraphEngine", DiagramKind::ThreatGraph, 15),
        ("model_sil", "Diagrams::TraceabilityRouteConflict", DiagramKind::Traceability, 4),
    ] {
        let (els, root) = shipped(model);
        let r = Resolver::new(&els);
        let d = find(&els, name);
        let cyber = CyberConfig::load(&root);
        let (g, issues) = build_graph_with(d, &els, &r, &cyber).expect("an IR");
        assert_eq!(g.kind, kind, "{name}");
        assert!(issues.is_empty(), "{name}: {issues:?}");
        assert!(g.nodes.len() >= min_nodes, "{name}: {} nodes", g.nodes.len());
        assert!(render_svg(&g, &no_links).is_ok(), "{name}");
        assert!(render_mermaid(&g, &no_links).is_some(), "{name}");
        assert!(render_plantuml(d, &els, None).is_some(), "{name}");
        let findings = validate(&els).findings;
        let errors: Vec<_> = findings.iter().filter(|f| f.code.starts_with('E')).collect();
        assert!(errors.is_empty(), "{model}: {errors:?}");
        assert!(!findings.iter().any(|f| f.code == "W417" || f.code == "W418"), "{model}: {findings:?}");
    }
}
