//! `REQ-TRS-VIS-003`/`-004`/`-005`: derived diagram content through the real
//! walker and validator. A `Diagram` with a `subject:` and no `shapes:` is
//! generated from the model; `include:`/`exclude:` filter it (`W417` for an
//! entry naming nothing, or filters on a manifest diagram); a subject of the
//! wrong type is `W418`. The generated IR for the fixture model is pinned by
//! golden JSON snapshots under `tests/vis_snapshots/derived/` — refresh with
//! `SYSCRIBE_UPDATE_SNAPSHOTS=1 cargo test -p syscribe-model --test vis_derive`
//! and review the diff.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::resolver::Resolver;
use syscribe_model::validator::{validate, Finding};
use syscribe_model::vis::{build_graph, DiagramGraph, EdgeKind, NodeKind};
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-vis-derive-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// The fixture: a `Sys` package with an abstract base, two definitions with
/// ports, a port definition, a connection definition and a composed
/// `PowerSystem` with inline part usages, a boundary port, a connection and a
/// binding, plus one file-per-usage child part.
fn fixture_model() -> PathBuf {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "Sys/_index.md", "---\ntype: Package\nname: Sys\n---\n");
    write(&root, "Sys/Base.md", "---\ntype: PartDef\nname: Base\nisAbstract: true\n---\n\nBase.\n");
    write(&root, "Sys/PowerPort.md", "---\ntype: PortDef\nname: PowerPort\n---\n\nPower port.\n");
    write(
        &root,
        "Sys/Engine.md",
        "---\ntype: PartDef\nname: Engine\nsupertype: Sys::Base\nfeatures:\n  - name: mass\n    typedBy: ScalarValues::Real\n    unit: SI::kg\n  - name: powerOut\n    type: Port\n    typedBy: Sys::PowerPort\n    direction: out\n---\n\nEngine.\n",
    );
    write(
        &root,
        "Sys/Motor.md",
        "---\ntype: PartDef\nname: Motor\nsupertype: Sys::Base\nfeatures:\n  - name: powerIn\n    type: Port\n    typedBy: Sys::PowerPort\n    direction: in\n---\n\nMotor.\n",
    );
    write(
        &root,
        "Sys/PowerLink.md",
        "---\ntype: ConnectionDef\nname: PowerLink\nends:\n  - name: source\n    typedBy: Sys::Engine\n  - name: target\n    typedBy: Sys::Motor\n---\n\nLink.\n",
    );
    write(
        &root,
        "Sys/PowerSystem.md",
        "---\ntype: PartDef\nname: PowerSystem\nfeatures:\n  - name: engine\n    typedBy: Sys::Engine\n  - name: motor\n    typedBy: Sys::Motor\n    multiplicity: \"2\"\n  - name: mainOut\n    type: Port\n    typedBy: Sys::PowerPort\n    direction: out\nconnections:\n  - typedBy: Sys::PowerLink\n    from: engine.powerOut\n    to: motor.powerIn\nbindingConnections:\n  - left: motor.powerIn\n    right: mainOut\n---\n\nPower system.\n",
    );
    write(&root, "Sys/PowerSystem/aux.md", "---\ntype: Part\nname: aux\ntypedBy: Sys::Motor\n---\n\nAuxiliary motor.\n");
    write(&root, "Diagrams/_index.md", "---\ntype: Package\nname: Diagrams\n---\n");
    root
}

fn add_diagram(root: &Path, name: &str, fm: &str) {
    write(root, &format!("Diagrams/{name}.md"), &format!("---\ntype: Diagram\nname: {name}\n{fm}---\n\n{name}.\n"));
}

fn graph_of(root: &Path, qname: &str) -> (DiagramGraph, Vec<Finding>) {
    let elements = walk_model(root).unwrap();
    let resolver = Resolver::new(&elements);
    let d = elements.iter().find(|e| e.qualified_name == qname).expect("diagram element");
    let (graph, _) = build_graph(d, &elements, &resolver).expect("an IR");
    let findings = validate(&elements)
        .findings
        .into_iter()
        .filter(|f| f.file.ends_with(&format!("{}.md", qname.replace("::", "/"))))
        .collect();
    (graph, findings)
}

fn codes(findings: &[Finding], code: &str) -> Vec<String> {
    findings.iter().filter(|f| f.code == code).map(|f| f.message.clone()).collect()
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

#[test]
fn derived_bdd_of_a_package_matches_its_golden_ir() {
    let root = fixture_model();
    add_diagram(&root, "SysBDD", "diagramKind: BDD\nsubject: Sys\n");
    let (g, findings) = graph_of(&root, "Diagrams::SysBDD");
    assert!(codes(&findings, "W417").is_empty() && codes(&findings, "W418").is_empty(), "{findings:?}");
    assert!(codes(&findings, "E405").is_empty());
    assert_eq!(g.nodes.iter().filter(|n| n.kind == NodeKind::Block).count(), 6);
    assert_eq!(g.edges.iter().filter(|e| e.kind == EdgeKind::Inheritance).count(), 2);
    assert_eq!(g.edges.iter().filter(|e| e.kind == EdgeKind::Composition).count(), 3);
    assert_eq!(g.edges.iter().filter(|e| e.kind == EdgeKind::Association).count(), 1);
    assert_snapshot("sys_bdd", &g);
}

#[test]
fn derived_ibd_of_a_partdef_matches_its_golden_ir() {
    let root = fixture_model();
    add_diagram(&root, "PowerIBD", "diagramKind: IBD\nsubject: Sys::PowerSystem\n");
    let (g, findings) = graph_of(&root, "Diagrams::PowerIBD");
    assert!(codes(&findings, "W417").is_empty() && codes(&findings, "W418").is_empty(), "{findings:?}");
    let boundary = g.roots().collect::<Vec<_>>();
    assert_eq!(boundary.len(), 1);
    assert_eq!(boundary[0].kind, NodeKind::Boundary);
    assert_eq!(g.nodes.iter().filter(|n| n.kind == NodeKind::Block).count(), 3, "engine, motor, aux");
    assert_eq!(g.nodes.iter().filter(|n| n.kind == NodeKind::Port).count(), 4, "mainOut, engine.powerOut, motor.powerIn, aux.powerIn");
    assert_eq!(g.edges.iter().map(|e| e.kind).collect::<Vec<_>>(), vec![EdgeKind::Connection, EdgeKind::Binding]);
    assert_snapshot("power_ibd", &g);
}

#[test]
fn pins_survive_regeneration_through_deterministic_ids() {
    let root = fixture_model();
    add_diagram(
        &root,
        "Pinned",
        "diagramKind: IBD\nsubject: Sys::PowerSystem\nlayout:\n  s-sys-powersystem-engine: {x: 40, y: 50}\n",
    );
    let (g, findings) = graph_of(&root, "Diagrams::Pinned");
    assert!(codes(&findings, "W416").is_empty());
    assert_eq!(g.pinned_ids(), vec!["s-sys-powersystem-engine"]);
}

#[test]
fn include_and_exclude_filter_and_a_stray_entry_is_w417() {
    let root = fixture_model();
    add_diagram(&root, "Some", "diagramKind: BDD\nsubject: Sys\ninclude: [Engine, Motor, Ghost]\nexclude: [Motor]\n");
    let (g, findings) = graph_of(&root, "Diagrams::Some");
    let blocks: Vec<&str> = g.nodes.iter().filter(|n| n.kind == NodeKind::Block).map(|n| n.element_ref.as_str()).collect();
    assert_eq!(blocks, vec!["Sys::Engine"]);
    let w417 = codes(&findings, "W417");
    assert_eq!(w417.len(), 1, "{findings:?}");
    assert!(w417[0].contains("'Ghost'"));
}

#[test]
fn filters_on_a_manifest_diagram_are_w417_and_ignored() {
    let root = fixture_model();
    add_diagram(&root, "Man", "diagramKind: BDD\nsubject: Sys\ninclude: [Engine]\nshapes:\n  a: Sys::Engine\n  b: Sys::Motor\n");
    let (g, findings) = graph_of(&root, "Diagrams::Man");
    assert_eq!(g.nodes.len(), 2, "the manifest wins and is not filtered");
    assert_eq!(codes(&findings, "W417").len(), 1);
}

#[test]
fn a_subject_of_the_wrong_type_is_w418_and_draws_nothing() {
    let root = fixture_model();
    add_diagram(&root, "Bad", "diagramKind: IBD\nsubject: Sys\n");
    let (g, findings) = graph_of(&root, "Diagrams::Bad");
    assert!(g.nodes.is_empty());
    let w418 = codes(&findings, "W418");
    assert_eq!(w418.len(), 1, "{findings:?}");
    assert!(w418[0].contains("Package"));
}

#[test]
fn a_derived_diagram_raises_no_w402_for_its_generated_refs() {
    // Generated nodes reference inline features (`Sys::PowerSystem::engine::powerOut`);
    // those are not elements, and the generator must not provoke the
    // manifest-ref warning on them.
    let root = fixture_model();
    add_diagram(&root, "Clean", "diagramKind: IBD\nsubject: Sys::PowerSystem\n");
    let (_g, findings) = graph_of(&root, "Diagrams::Clean");
    assert!(codes(&findings, "W402").is_empty(), "{findings:?}");
    assert!(codes(&findings, "W403").is_empty());
}
