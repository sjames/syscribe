//! `REQ-TRS-VIS-030` (GH #337): external actors. A part usage marked `role: external` is an actor outside the
//! product: derived IBDs and BDDs draw it «actor» (a `role: environment` part «environment», dashed), and in a
//! context composite (a part holding at least one actor) every port of a product part must reach something
//! (`W996`). External parts are no evidence, like environment parts.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::resolver::Resolver;
use syscribe_model::validator::{validate, Finding};
use syscribe_model::vis::style::node_style;
use syscribe_model::vis::{build_graph, DiagramGraph};
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-vis-context-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// An inverter with two boundary ports, a PV array and a grid as actors, and a test bench part.
fn model(connections: &str) -> PathBuf {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "Sys/_index.md", "---\ntype: Package\nname: Sys\n---\n");
    write(&root, "Sys/Inverter.md", "---\ntype: PartDef\nname: Inverter\ndomain: system\nfeatures:\n  - {name: pvIn, type: Port, direction: in}\n  - {name: gridOut, type: Port, direction: out}\n---\n");
    write(&root, "Sys/PvArray.md", "---\ntype: PartDef\nname: PvArray\nfeatures:\n  - {name: dcOut, type: Port, direction: out}\n---\n");
    write(&root, "Sys/Grid.md", "---\ntype: PartDef\nname: Grid\nfeatures:\n  - {name: acIn, type: Port, direction: in}\n---\n");
    write(&root, "Sys/Bench.md", "---\ntype: PartDef\nname: Bench\n---\n");
    write(
        &root,
        "Sys/Context.md",
        &format!("---\ntype: PartDef\nname: Context\ndomain: system\nfeatures:\n  - {{name: inverter, typedBy: Sys::Inverter}}\n  - {{name: pv, typedBy: Sys::PvArray, role: external}}\n  - {{name: grid, typedBy: Sys::Grid, role: external}}\n  - {{name: bench, typedBy: Sys::Bench, role: environment}}\nconnections:\n{connections}---\n"),
    );
    write(&root, "Diagrams/_index.md", "---\ntype: Package\nname: Diagrams\n---\n");
    write(&root, "Diagrams/Ibd.md", "---\ntype: Diagram\nname: Ibd\ndiagramKind: IBD\nsubject: Sys::Context\n---\n\nIBD.\n");
    write(&root, "Diagrams/Bdd.md", "---\ntype: Diagram\nname: Bdd\ndiagramKind: BDD\nsubject: Sys::Context\n---\n\nBDD.\n");
    root
}

const BOTH: &str = "  - {from: pv.dcOut, to: inverter.pvIn}\n  - {from: inverter.gridOut, to: grid.acIn}\n";

fn graph(root: &Path, q: &str) -> DiagramGraph {
    let elements = walk_model(root).unwrap();
    let resolver = Resolver::new(&elements);
    let d = elements.iter().find(|e| e.qualified_name == q).unwrap();
    build_graph(d, &elements, &resolver).unwrap().0
}

fn hits(root: &Path, code: &str) -> Vec<Finding> {
    validate(&walk_model(root).unwrap()).findings.into_iter().filter(|f| f.code == code).collect()
}

#[test]
fn the_context_ibd_draws_actors_outside_the_system_and_the_environment_dashed() {
    let g = graph(&model(BOTH), "Diagrams::Ibd");
    let st = |id: &str| g.node(id).unwrap_or_else(|| panic!("{id}")).stereotype.clone();
    assert_eq!(st("s-sys-context-inverter").as_deref(), Some("part"));
    assert_eq!(st("s-sys-context-pv").as_deref(), Some("actor"));
    assert_eq!(st("s-sys-context-grid").as_deref(), Some("actor"));
    assert_eq!(st("s-sys-context-bench").as_deref(), Some("environment"));
    assert!(node_style(g.node("s-sys-context-bench").unwrap()).dashed);
    assert!(!node_style(g.node("s-sys-context-pv").unwrap()).dashed);
    // actors sit beside the system, not in it, and connect to its ports
    assert_eq!(g.node("s-sys-context-pv").unwrap().parent, g.node("s-sys-context-inverter").unwrap().parent);
    assert!(g.edges.iter().any(|e| e.source == "s-sys-context-pv-dcout" && e.target == "s-sys-context-inverter-pvin"));
}

#[test]
fn the_context_bdd_marks_actor_definitions() {
    let g = graph(&model(BOTH), "Diagrams::Bdd");
    assert_eq!(g.node("s-sys-pvarray").expect("an actor block").stereotype.as_deref(), Some("actor"));
    assert_eq!(g.node("s-sys-inverter").unwrap().stereotype.as_deref(), Some("part def"));
}

#[test]
fn an_unconnected_system_port_in_a_context_is_w996() {
    let root = model("  - {from: pv.dcOut, to: inverter.pvIn}\n");
    let f = hits(&root, "W996");
    assert_eq!(f.len(), 1, "{f:?}");
    assert!(f[0].message.contains("'gridOut'") && f[0].message.contains("'inverter'"), "{}", f[0].message);
    assert!(hits(&model(BOTH), "W996").is_empty());
}

#[test]
fn without_an_actor_there_is_no_context_and_no_w996() {
    let root = model("");
    let ctx = std::fs::read_to_string(root.join("Sys/Context.md")).unwrap();
    write(&root, "Sys/Context.md", &ctx.replace(", role: external", ""));
    assert!(hits(&root, "W996").is_empty());
}

#[test]
fn external_is_a_role_and_anything_else_is_still_e992() {
    let root = model(BOTH);
    assert!(hits(&root, "E992").is_empty());
    let ctx = std::fs::read_to_string(root.join("Sys/Context.md")).unwrap();
    write(&root, "Sys/Context.md", &ctx.replacen("role: external", "role: outside", 1));
    let f = hits(&root, "E992");
    assert_eq!(f.len(), 1);
    assert!(f[0].message.contains("environment, external"), "{}", f[0].message);
}

#[test]
fn a_context_is_the_root_of_its_context_and_needs_no_user() {
    let root = model(BOTH);
    let w007: Vec<String> = hits(&root, "W007").into_iter().map(|f| f.message).collect();
    assert!(!w007.iter().any(|m| m.contains("'Sys::Context'")), "{w007:?}");
}

#[test]
fn w996_finds_inherited_child_and_portdef_ports_and_child_part_usages() {
    let root = model(BOTH);
    // an inherited port, a child `Port` file and a port typed by a PortDef on the inverter
    write(&root, "Sys/Base.md", "---\ntype: PartDef\nname: Base\nfeatures:\n  - {name: earth, type: Port}\n---\n");
    write(&root, "Sys/Dc.md", "---\ntype: PortDef\nname: Dc\n---\n");
    write(&root, "Sys/Inverter.md", "---\ntype: PartDef\nname: Inverter\nsupertype: Sys::Base\nfeatures:\n  - {name: pvIn, type: Port, direction: in}\n  - {name: gridOut, type: Port, direction: out}\n  - {name: aux, typedBy: Sys::Dc}\n---\n");
    write(&root, "Sys/Inverter/extra.md", "---\ntype: Port\nname: extra\n---\n");
    // a second product part as a child `Part` file of the context
    write(&root, "Sys/Context/meter.md", "---\ntype: Part\nname: meter\ntypedBy: Sys::Grid\n---\n");
    let msgs: Vec<String> = hits(&root, "W996").into_iter().map(|f| f.message).collect();
    for want in ["port 'earth' of 'inverter'", "port 'extra' of 'inverter'", "port 'aux' of 'inverter'", "port 'acIn' of 'meter'"] {
        assert!(msgs.iter().any(|m| m.contains(want)), "no {want}: {msgs:?}");
    }
    assert_eq!(msgs.len(), 4, "{msgs:?}");
}

#[test]
fn w996_reads_flows_bindings_ends_and_qualified_chains() {
    let conns = "  - {from: pv::dcOut, to: inverter::pvIn}\n";
    let root = model(conns);
    let ctx = std::fs::read_to_string(root.join("Sys/Context.md")).unwrap();
    std::fs::write(root.join("Sys/Context.md"), ctx.replacen("connections:\n", "flowConnections:\n  - {from: inverter.gridOut, to: grid.acIn}\nconnections:\n", 1)).unwrap();
    assert!(hits(&root, "W996").is_empty(), "{:?}", hits(&root, "W996"));
}

#[test]
fn a_connection_to_an_actor_that_is_not_there_reaches_nothing() {
    // as after `--config` drops the PV array: its connection no longer reaches the inverter's PV input
    let root = model(BOTH);
    let ctx = std::fs::read_to_string(root.join("Sys/Context.md")).unwrap();
    std::fs::write(root.join("Sys/Context.md"), ctx.replace("  - {name: pv, typedBy: Sys::PvArray, role: external}\n", "")).unwrap();
    let msgs: Vec<String> = hits(&root, "W996").into_iter().map(|f| f.message).collect();
    assert!(msgs.len() == 1 && msgs[0].contains("port 'pvIn' of 'inverter'"), "{msgs:?}");
}

#[test]
fn dot_plantuml_and_mermaid_mark_actors_and_dash_the_environment() {
    let root = model(BOTH);
    let elements = walk_model(&root).unwrap();
    let d = elements.iter().find(|e| e.qualified_name == "Diagrams::Ibd").unwrap();
    let g = graph(&root, "Diagrams::Ibd");
    let puml = syscribe_model::plantuml::render_plantuml(d, &elements, None).unwrap();
    assert!(puml.contains("<<actor>>") && puml.contains("<<environment>>") && puml.contains("line.dashed"), "{puml}");
    let mmd = syscribe_model::vis::render_mermaid(&g, &|_| None).unwrap();
    assert!(mmd.contains("«actor»<br/>") && mmd.contains("«environment»<br/>"), "{mmd}");
    let dot = syscribe_model::vis::render_dot(&g);
    let bench = dot.lines().find(|l| l.contains("bench") && l.contains("label=")).unwrap();
    assert!(bench.contains("dashed"), "{bench}");
    let pv = dot.lines().find(|l| l.contains("label=") && l.contains("pv :")).unwrap();
    assert!(!pv.contains("dashed"), "{pv}");
}

#[test]
fn a_gated_port_is_judged_by_what_is_written() {
    // the PV input is gated with the PV array: inline features are not projected, so its written connection counts
    let root = model(BOTH);
    write(&root, "Sys/Inverter.md", "---\ntype: PartDef\nname: Inverter\ndomain: system\nfeatures:\n  - {name: pvIn, type: Port, direction: in, appliesWhen: Sys::Pv}\n  - {name: gridOut, type: Port, direction: out}\n---\n");
    let ctx = std::fs::read_to_string(root.join("Sys/Context.md")).unwrap();
    std::fs::write(root.join("Sys/Context.md"), ctx.replace("  - {name: pv, typedBy: Sys::PvArray, role: external}\n", "")).unwrap();
    assert!(hits(&root, "W996").is_empty(), "{:?}", hits(&root, "W996"));
}
