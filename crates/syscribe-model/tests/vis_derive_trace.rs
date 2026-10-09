//! `REQ-TRS-VIS-020`/`-022`: the derived Requirement and Allocation diagrams
//! through the real walker and validator. A `Diagram` with `diagramKind:
//! Requirement` and a package subject draws the requirements under it with
//! their derive/refine/satisfy/verify legs; one with `diagramKind: Allocation`
//! draws every allocation pair under its subject as `«allocate»` arrows between
//! a logical and a physical swimlane. The generated IR for the fixture model is
//! pinned by golden JSON snapshots under `tests/vis_snapshots/derived/` —
//! refresh with `SYSCRIBE_UPDATE_SNAPSHOTS=1 cargo test -p syscribe-model
//! --test vis_derive_trace` and review the diff.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::element::RawElement;
use syscribe_model::plantuml::render_plantuml;
use syscribe_model::resolver::Resolver;
use syscribe_model::validator::{validate, Finding};
use syscribe_model::vis::{build_graph, render_mermaid, render_svg, DiagramGraph, EdgeKind, NodeKind};
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-vis-trace-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// The fixture: a `Reqs` package with a top requirement, a `RequirementDef`
/// owning two derived children (one of which also refines the other), an
/// `Arch` package with a `PartDef` satisfying one child, an `ActionDef` and an
/// ECU, a `Tests` package with a `TestCase` verifying the other child, and an
/// `Allocations` package with an `Allocation` element (`features:` pairs, one
/// end unresolved), an `AllocationDef` (`allocations:`) and a `PartDef` with
/// its own `allocatedTo:`.
fn fixture_model() -> PathBuf {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "Reqs/_index.md", "---\ntype: Package\nname: Reqs\n---\n");
    write(&root, "Reqs/Top.md", "---\ntype: Requirement\nid: REQ-TR-001\nname: \"Top requirement\"\nstatus: approved\n---\n\nTop.\n");
    write(&root, "Reqs/Safety.md", "---\ntype: RequirementDef\nname: Safety\nisAbstract: true\n---\n\nSafety requirements.\n");
    write(
        &root,
        "Reqs/Safety/Left.md",
        "---\ntype: Requirement\nid: REQ-TR-002\nname: \"Left requirement\"\nstatus: draft\nderivedFrom: [REQ-TR-001]\n---\n\nLeft.\n",
    );
    write(
        &root,
        "Reqs/Safety/Right.md",
        "---\ntype: Requirement\nid: REQ-TR-003\nname: \"Right requirement\"\nderivedFrom: [Reqs::Top]\nrefines: [REQ-TR-002]\n---\n\nRight.\n",
    );
    write(&root, "Arch/_index.md", "---\ntype: Package\nname: Arch\n---\n");
    write(&root, "Arch/Ctrl.md", "---\ntype: PartDef\nname: Ctrl\nsatisfies: [REQ-TR-002]\n---\n\nController.\n");
    write(&root, "Arch/Ecu.md", "---\ntype: PartDef\nname: Ecu\n---\n\nECU.\n");
    write(&root, "Arch/Nav.md", "---\ntype: ActionDef\nname: Nav\n---\n\nNavigate.\n");
    write(&root, "Tests/_index.md", "---\ntype: Package\nname: Tests\n---\n");
    write(
        &root,
        "Tests/CtrlTest.md",
        "---\ntype: TestCase\nid: TC-TR-001\nname: \"Controller test\"\nstatus: active\ntestLevel: L2\nverifies: [REQ-TR-003]\n---\n\nTest.\n",
    );
    write(&root, "Allocations/_index.md", "---\ntype: Package\nname: Allocations\n---\n");
    write(
        &root,
        "Allocations/FnAlloc.md",
        "---\ntype: Allocation\nname: FnAlloc\nfeatures:\n  - name: navToEcu\n    type: Allocation\n    allocatedFrom: Arch::Nav\n    allocatedTo: Arch::Ecu\n  - name: ctrlToGhost\n    type: Allocation\n    allocatedFrom: Arch::Ctrl\n    allocatedTo: Hw::Ghost\n---\n\nFunction allocation.\n",
    );
    write(
        &root,
        "Allocations/FnDef.md",
        "---\ntype: AllocationDef\nname: FnDef\nallocations:\n  - name: navToCtrl\n    allocatedFrom: Arch::Nav\n    allocatedTo: Arch::Ctrl\n---\n\nAllocation definition.\n",
    );
    write(&root, "Allocations/CtrlSw.md", "---\ntype: PartDef\nname: CtrlSw\nallocatedTo: [Arch::Ecu]\n---\n\nControl software.\n");
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

fn edges(g: &DiagramGraph, kind: EdgeKind) -> Vec<(String, String, Option<String>)> {
    g.edges.iter().filter(|e| e.kind == kind).map(|e| (e.source.clone(), e.target.clone(), e.label.clone())).collect()
}

fn ids(g: &DiagramGraph, kind: NodeKind) -> Vec<String> {
    g.nodes.iter().filter(|n| n.kind == kind).map(|n| n.id.clone()).collect()
}

fn snapshot_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/vis_snapshots/derived")
}

/// Compare `graph` against the committed golden JSON `name.json`.
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

// ── Requirement (REQ-TRS-VIS-020) ────────────────────────────────────────────

#[test]
fn derived_requirement_diagram_of_a_package_matches_its_golden_ir() {
    let root = fixture_model();
    add_diagram(&root, "ReqsD", "diagramKind: Requirement\nsubject: Reqs\n");
    let d = derive(&root, "Diagrams::ReqsD");
    let g = &d.graph;
    assert!(codes(&d.findings, "W417").is_empty() && codes(&d.findings, "W418").is_empty(), "{:?}", d.findings);
    assert!(g.derived);
    // One requirement node per Requirement/RequirementDef under the subject, any depth.
    assert_eq!(ids(g, NodeKind::Requirement), vec!["s-reqs-safety", "s-reqs-safety-left", "s-reqs-safety-right", "s-reqs-top"]);
    // Labelled by name, with id and status compartment lines when present.
    let top = g.node("s-reqs-top").unwrap();
    assert_eq!((top.label.as_str(), top.stereotype.as_deref(), top.element_type.as_deref()), ("Top requirement", Some("requirement"), Some("Requirement")));
    assert_eq!(g.node("s-reqs-top-compartment").unwrap().lines, vec!["id = REQ-TR-001", "status = approved"]);
    assert_eq!(g.node("s-reqs-safety-right-compartment").unwrap().lines, vec!["id = REQ-TR-003"], "no status: one line");
    assert!(g.node("s-reqs-safety-compartment").is_none(), "a RequirementDef with neither id nor status has no compartment");
    assert_eq!(g.node("s-reqs-safety").unwrap().stereotype.as_deref(), Some("requirement def"));
    // Derive runs child → parent, whether the target is named by id or by qualified name.
    assert_eq!(
        edges(g, EdgeKind::Derive),
        vec![("s-reqs-safety-left".into(), "s-reqs-top".into(), None), ("s-reqs-safety-right".into(), "s-reqs-top".into(), None)]
    );
    // Refine runs from the refining requirement to the refined one.
    assert_eq!(edges(g, EdgeKind::Refine), vec![("s-reqs-safety-right".into(), "s-reqs-safety-left".into(), None)]);
    // Containment runs from the RequirementDef to each requirement it owns.
    assert_eq!(
        edges(g, EdgeKind::Containment),
        vec![("s-reqs-safety".into(), "s-reqs-safety-left".into(), None), ("s-reqs-safety".into(), "s-reqs-safety-right".into(), None)]
    );
    // Satisfy runs from a Block context node drawn with its real type's stereotype.
    assert_eq!(edges(g, EdgeKind::Satisfy), vec![("s-arch-ctrl".into(), "s-reqs-safety-left".into(), None)]);
    let ctrl = g.node("s-arch-ctrl").unwrap();
    assert_eq!((ctrl.kind, ctrl.stereotype.as_deref(), ctrl.element_type.as_deref()), (NodeKind::Block, Some("part def"), Some("PartDef")));
    // Verify runs from a TestCase context node.
    assert_eq!(edges(g, EdgeKind::Verify), vec![("s-tests-ctrltest".into(), "s-reqs-safety-right".into(), None)]);
    let tc = g.node("s-tests-ctrltest").unwrap();
    assert_eq!((tc.kind, tc.stereotype.as_deref(), tc.label.as_str()), (NodeKind::TestCase, Some("test case"), "Controller test"));
    assert_eq!(g.edges.len(), 7);
    // Deterministic ids and the top-to-bottom hints with parents above.
    assert!(g.edges.iter().any(|e| e.id == "e-refine-s-reqs-safety-right-s-reqs-safety-left"));
    assert_eq!(g.layout_hints.reversed_kinds, vec![EdgeKind::Derive, EdgeKind::Satisfy, EdgeKind::Verify, EdgeKind::Refine]);
    assert_snapshot("reqs", g);
}

#[test]
fn requirement_def_and_requirement_subjects_root_the_tree() {
    let root = fixture_model();
    add_diagram(&root, "SafetyD", "diagramKind: Requirement\nsubject: Reqs::Safety\n");
    let d = derive(&root, "Diagrams::SafetyD");
    assert!(codes(&d.findings, "W418").is_empty());
    assert_eq!(ids(&d.graph, NodeKind::Requirement), vec!["s-reqs-safety", "s-reqs-safety-left", "s-reqs-safety-right"]);
    assert!(d.graph.node("s-reqs-top").is_none());
    assert!(edges(&d.graph, EdgeKind::Derive).is_empty(), "the parent is outside the subject");
    assert_eq!(edges(&d.graph, EdgeKind::Containment).len(), 2);

    add_diagram(&root, "OneD", "diagramKind: Requirement\nsubject: REQ-TR-002\n");
    let d = derive(&root, "Diagrams::OneD");
    assert!(codes(&d.findings, "W418").is_empty());
    assert_eq!(ids(&d.graph, NodeKind::Requirement), vec!["s-reqs-safety-left"]);
    assert_eq!(d.graph.edges.iter().map(|e| e.kind).collect::<Vec<_>>(), vec![EdgeKind::Satisfy], "only its own satisfier is drawn");
}

#[test]
fn requirement_filters_apply_to_requirements_and_context_nodes_and_a_stray_entry_is_w417() {
    let root = fixture_model();
    add_diagram(&root, "SomeD", "diagramKind: Requirement\nsubject: Reqs\ninclude: [REQ-TR-001, Left, Tests::CtrlTest, Nobody]\n");
    let d = derive(&root, "Diagrams::SomeD");
    assert_eq!(ids(&d.graph, NodeKind::Requirement), vec!["s-reqs-safety-left", "s-reqs-top"], "by stable id and by short name");
    assert!(d.graph.node("s-tests-ctrltest").is_none(), "an included test case whose requirement is not on the diagram draws nothing");
    assert_eq!(d.graph.edges.iter().map(|e| e.kind).collect::<Vec<_>>(), vec![EdgeKind::Derive]);
    let w417 = codes(&d.findings, "W417");
    assert_eq!(w417.len(), 1, "{:?}", d.findings);
    assert!(w417[0].contains("'Nobody'"));

    add_diagram(&root, "LessD", "diagramKind: Requirement\nsubject: Reqs\nexclude: [Arch::Ctrl, Safety]\n");
    let d = derive(&root, "Diagrams::LessD");
    assert!(codes(&d.findings, "W417").is_empty());
    assert!(d.graph.node("s-arch-ctrl").is_none() && d.graph.node("s-reqs-safety").is_none());
    assert!(d.graph.edges.iter().all(|e| !matches!(e.kind, EdgeKind::Satisfy | EdgeKind::Containment)));
}

#[test]
fn requirement_subject_of_the_wrong_type_is_w418_and_draws_nothing() {
    let root = fixture_model();
    add_diagram(&root, "BadD", "diagramKind: Requirement\nsubject: Arch::Ecu\n");
    let d = derive(&root, "Diagrams::BadD");
    assert!(d.graph.nodes.is_empty());
    let w418 = codes(&d.findings, "W418");
    assert_eq!(w418.len(), 1, "{:?}", d.findings);
    assert!(w418[0].contains("PartDef"));
}

#[test]
fn requirement_writers_accept_the_derived_graph() {
    let root = fixture_model();
    add_diagram(&root, "ReqsD", "diagramKind: Requirement\nsubject: Reqs\n");
    let d = derive(&root, "Diagrams::ReqsD");
    let m = render_mermaid(&d.graph, &no_links).expect("Mermaid class diagram");
    assert!(m.starts_with("classDiagram\n"));
    assert!(m.contains("  class s_reqs_top[\"Top requirement\"] {\n    <<requirement>>\n    +id = REQ-TR-001\n    +status = approved\n  }\n"), "{m}");
    assert!(m.contains("  s_reqs_safety_left ..> s_reqs_top : «deriveReqt»\n"), "{m}");
    assert!(m.contains("  s_tests_ctrltest ..> s_reqs_safety_right : «verify»\n"), "{m}");
    let s = render_svg(&d.graph, &no_links).expect("SVG via the embedded ELK");
    assert!(s.contains("<g id=\"s-reqs-top\" class=\"requirement Requirement\""), "{s}");
    assert!(s.contains("<path id=\"e-derive-s-reqs-safety-left-s-reqs-top\" class=\"edge derive\"") && s.contains("stroke-dasharray=\"6,3\""), "{s}");
    assert!(s.contains("«deriveReqt»") && s.contains("«satisfy»") && s.contains("«verify»") && s.contains("«refine»"), "{s}");
    assert!(s.contains("fill-opacity=\"0.15\""), "requirement and test-case boxes carry a header band");
    let p = render_plantuml(&d.diagram, &d.elements, None).expect("PlantUML maps the Requirement kind");
    assert!(p.contains("class \"Top requirement\" as s_reqs_top <<requirement>>  {\n  id = REQ-TR-001\n  status = approved\n}\n"), "{p}");
    assert!(p.contains("class \"Controller test\" as s_tests_ctrltest <<test case>> \n"), "{p}");
    assert!(!p.contains("compartment"), "a compartment is never a class of its own: {p}");
    assert!(p.contains("s_reqs_safety_right ..> s_reqs_safety_left : refines\n"), "{p}");
}

// ── Allocation (REQ-TRS-VIS-022) ─────────────────────────────────────────────

#[test]
fn derived_allocation_diagram_of_a_package_matches_its_golden_ir() {
    let root = fixture_model();
    add_diagram(&root, "AllocD", "diagramKind: Allocation\nsubject: Allocations\n");
    let d = derive(&root, "Diagrams::AllocD");
    let g = &d.graph;
    assert!(codes(&d.findings, "W417").is_empty() && codes(&d.findings, "W418").is_empty(), "{:?}", d.findings);
    // Two lanes named after the subject's id.
    let lanes: Vec<(&str, &str)> = g.roots().map(|n| (n.id.as_str(), n.label.as_str())).collect();
    assert_eq!(lanes, vec![("s-allocations-logical", "Logical"), ("s-allocations-physical", "Physical")]);
    assert!(g.roots().all(|n| n.kind == NodeKind::Swimlane && n.element_ref == "Allocations"));
    // Membership: sources in the logical lane, targets in the physical one, each once.
    let members = |lane: &str| g.children_of(lane).map(|n| n.element_ref.clone()).collect::<Vec<_>>();
    assert_eq!(members("s-allocations-logical"), vec!["Allocations::CtrlSw", "Arch::Nav", "Arch::Ctrl"]);
    assert_eq!(members("s-allocations-physical"), vec!["Arch::Ecu", "Hw::Ghost", "Arch::Ctrl"]);
    // Real type stereotypes; the same element in both lanes; the unresolved end dashed.
    assert_eq!(g.node("s-arch-nav").unwrap().stereotype.as_deref(), Some("action def"));
    assert_eq!(g.node("s-arch-ctrl").unwrap().parent.as_deref(), Some("s-allocations-logical"));
    assert_eq!(g.node("s-arch-ctrl-physical").unwrap().parent.as_deref(), Some("s-allocations-physical"));
    let ghost = g.node("s-hw-ghost").unwrap();
    assert!(!ghost.resolved && ghost.element_type.is_none() && ghost.label == "Hw::Ghost");
    // One allocate edge per pair, labelled by the usage name when it has one.
    assert_eq!(
        edges(g, EdgeKind::Allocation),
        vec![
            ("s-allocations-ctrlsw".into(), "s-arch-ecu".into(), None),
            ("s-arch-nav".into(), "s-arch-ecu".into(), Some("navToEcu".into())),
            ("s-arch-ctrl".into(), "s-hw-ghost".into(), Some("ctrlToGhost".into())),
            ("s-arch-nav".into(), "s-arch-ctrl-physical".into(), Some("navToCtrl".into())),
        ]
    );
    assert_eq!(g.edges.len(), 4);
    assert_eq!(g.edges[1].element_ref.as_deref(), Some("Allocations::FnAlloc"));
    assert!(g.edges.iter().any(|e| e.id == "e-allocation-s-arch-nav-s-arch-ecu-navtoecu"));
    assert!(g.layout_hints.hierarchical);
    assert_snapshot("alloc", g);
}

#[test]
fn allocation_filters_apply_to_the_end_elements_and_a_stray_entry_is_w417() {
    let root = fixture_model();
    add_diagram(&root, "SomeA", "diagramKind: Allocation\nsubject: Allocations\ninclude: [Nav, Arch::Ecu, Hw::Ghost, Nobody]\n");
    let d = derive(&root, "Diagrams::SomeA");
    let g = &d.graph;
    assert_eq!(g.children_of("s-allocations-logical").map(|n| n.id.as_str()).collect::<Vec<_>>(), vec!["s-arch-nav"]);
    assert_eq!(g.children_of("s-allocations-physical").map(|n| n.id.as_str()).collect::<Vec<_>>(), vec!["s-arch-ecu", "s-hw-ghost"]);
    assert_eq!(g.edges.len(), 1, "only Nav → Ecu has both ends kept: {:?}", g.edges);
    let w417 = codes(&d.findings, "W417");
    assert_eq!(w417.len(), 1, "{:?}", d.findings);
    assert!(w417[0].contains("'Nobody'"));

    add_diagram(&root, "LessA", "diagramKind: Allocation\nsubject: Allocations\nexclude: [Arch::Nav]\n");
    let d = derive(&root, "Diagrams::LessA");
    assert!(codes(&d.findings, "W417").is_empty());
    assert!(d.graph.node("s-arch-nav").is_none());
    assert_eq!(d.graph.edges.len(), 2);
}

#[test]
fn allocation_subject_of_the_wrong_type_is_w418_and_draws_nothing() {
    let root = fixture_model();
    add_diagram(&root, "BadA", "diagramKind: Allocation\nsubject: Arch::Ecu\n");
    let d = derive(&root, "Diagrams::BadA");
    assert!(d.graph.nodes.is_empty());
    let w418 = codes(&d.findings, "W418");
    assert_eq!(w418.len(), 1, "{:?}", d.findings);
    assert!(w418[0].contains("PartDef"));

    add_diagram(&root, "DefA", "diagramKind: Allocation\nsubject: Allocations::FnDef\n");
    let d = derive(&root, "Diagrams::DefA");
    assert!(codes(&d.findings, "W418").is_empty(), "an AllocationDef subject is valid");
    assert_eq!(d.graph.edges.len(), 1);
}

#[test]
fn allocation_writers_accept_the_derived_graph() {
    let root = fixture_model();
    add_diagram(&root, "AllocD", "diagramKind: Allocation\nsubject: Allocations\n");
    let d = derive(&root, "Diagrams::AllocD");
    let m = render_mermaid(&d.graph, &no_links).expect("Mermaid flowchart");
    assert!(m.starts_with("flowchart LR\n"));
    assert!(m.contains("  subgraph s_allocations_logical[\"Logical\"]\n"), "{m}");
    assert!(m.contains("  subgraph s_allocations_physical[\"Physical\"]\n"), "{m}");
    assert!(m.contains("    s_arch_nav[\"Nav\"]\n"), "{m}");
    assert!(m.contains("  s_allocations_ctrlsw -.->|«allocate»| s_arch_ecu\n"), "{m}");
    assert!(m.contains("  s_arch_nav -.->|navToEcu| s_arch_ecu\n"), "{m}");
    let s = render_svg(&d.graph, &no_links).expect("SVG via the embedded ELK");
    assert!(s.contains("<g id=\"s-allocations-logical\" class=\"swimlane\""), "{s}");
    assert!(s.contains("class=\"edge allocation\"") && s.contains("stroke-dasharray=\"8,4\"") && s.contains("«allocate»"), "{s}");
    assert!(s.contains("<g id=\"s-hw-ghost\" class=\"block unresolved\"") && s.contains("stroke-dasharray=\"4,3\""), "{s}");
    let puml = render_plantuml(&d.diagram, &d.elements, None).expect("Allocation renders to PlantUML");
    assert!(puml.starts_with("@startuml") && puml.contains("..>") && puml.trim_end().ends_with("@enduml"), "{puml}");
}
