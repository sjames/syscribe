//! `REQ-TRS-VIS-029` (GH #336): grouping and containment derived when an IBD is generated. `depth:` nests
//! parts; a deployment package is drawn inside the hardware it is allocated to; `appliesWhen:` badges a part
//! optional and a safety level outlines it, through the node mark; `groupBy: domain` frames the parts by
//! `domain:`; `--config` drops inactive parts. Without these fields an IBD draws as before.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::resolver::Resolver;
use syscribe_model::validator::validate;
use syscribe_model::vis::{build_graph, DiagramGraph, NodeKind, Tone};
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-vis-grouping-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// An inverter: a control board holding an MCU, firmware (a deployment package) allocated to the MCU, a
/// SIL 2 output stage, and an arc-fault detector that applies only with the `Afci` feature.
fn model(diagram_fm: &str) -> PathBuf {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "Features/_index.md", "---\ntype: Package\nname: Features\n---\n");
    write(&root, "Features/Afci.md", "---\ntype: FeatureDef\nid: FEAT-AFCI\nname: Afci\n---\n");
    write(&root, "Inv/_index.md", "---\ntype: Package\nname: Inv\n---\n");
    write(&root, "Inv/Mcu.md", "---\ntype: PartDef\nname: Mcu\ndomain: hardware\n---\n");
    write(
        &root,
        "Inv/Board.md",
        "---\ntype: PartDef\nname: Board\ndomain: hardware\nfeatures:\n  - {name: mcu, typedBy: Inv::Mcu}\n  - {name: pwm, type: Port, direction: out}\n---\n",
    );
    write(&root, "Inv/Task.md", "---\ntype: PartDef\nname: Task\ndomain: software\n---\n");
    write(
        &root,
        "Inv/Firmware.md",
        "---\ntype: PartDef\nname: Firmware\ndomain: software\nisDeploymentPackage: true\nfeatures:\n  - {name: task, typedBy: Inv::Task}\n---\n",
    );
    write(&root, "Inv/Stage.md", "---\ntype: PartDef\nname: Stage\ndomain: hardware\nsilLevel: 2\nfeatures:\n  - {name: gate, type: Port, direction: in}\n---\n");
    write(&root, "Inv/Arc.md", "---\ntype: PartDef\nname: Arc\ndomain: hardware\nappliesWhen: Features::Afci\n---\n");
    write(
        &root,
        "Inv/Inverter.md",
        "---\ntype: PartDef\nname: Inverter\ndomain: system\nfeatures:\n  - {name: board, typedBy: Inv::Board}\n  - {name: firmware, typedBy: Inv::Firmware}\n  - {name: stage, typedBy: Inv::Stage}\n  - {name: arc, typedBy: Inv::Arc}\nconnections:\n  - {from: board.pwm, to: stage.gate}\n---\n",
    );
    write(&root, "Inv/FwToMcu.md", "---\ntype: Allocation\nname: FwToMcu\nallocatedFrom: [Inv::Firmware]\nallocatedTo: [Inv::Mcu]\n---\n");
    write(&root, "Diagrams/_index.md", "---\ntype: Package\nname: Diagrams\n---\n");
    write(&root, "Diagrams/Ibd.md", &format!("---\ntype: Diagram\nname: Ibd\ndiagramKind: IBD\nsubject: Inv::Inverter\n{diagram_fm}---\n\nIBD.\n"));
    root
}

fn graph(root: &Path) -> DiagramGraph {
    let elements = walk_model(root).unwrap();
    let resolver = Resolver::new(&elements);
    let d = elements.iter().find(|e| e.qualified_name == "Diagrams::Ibd").unwrap();
    build_graph(d, &elements, &resolver).unwrap().0
}

fn parent<'a>(g: &'a DiagramGraph, id: &str) -> Option<&'a str> {
    g.node(id).unwrap_or_else(|| panic!("no node {id}: {:?}", g.nodes.iter().map(|n| &n.id).collect::<Vec<_>>())).parent.as_deref()
}

fn parents_precede_children(g: &DiagramGraph) {
    for (i, n) in g.nodes.iter().enumerate() {
        if let Some(p) = &n.parent {
            let pi = g.nodes.iter().position(|m| &m.id == p).expect("parent exists");
            assert!(pi < i, "{} comes before its parent {p}", n.id);
        }
    }
}

#[test]
fn firmware_is_drawn_inside_the_mcu_inside_the_board() {
    let g = graph(&model("depth: 2\n"));
    assert_eq!(parent(&g, "s-inv-inverter-board-mcu"), Some("s-inv-inverter-board"));
    assert_eq!(parent(&g, "s-inv-inverter-firmware"), Some("s-inv-inverter-board-mcu"), "deployment package inside its hardware");
    assert_eq!(parent(&g, "s-inv-inverter-firmware-task"), Some("s-inv-inverter-firmware"), "its own parts follow it");
    parents_precede_children(&g);
}

#[test]
fn at_depth_one_the_firmware_goes_into_the_board_that_holds_the_mcu() {
    let g = graph(&model(""));
    assert!(g.node("s-inv-inverter-board-mcu").is_none(), "depth 1 draws no nested parts");
    assert_eq!(parent(&g, "s-inv-inverter-firmware"), Some("s-inv-inverter-board"));
    parents_precede_children(&g);
}

#[test]
fn safety_level_outlines_and_applies_when_badges_optional() {
    let g = graph(&model(""));
    let stage = g.node("s-inv-inverter-stage").unwrap().mark.clone().expect("stage is marked");
    assert_eq!(stage.badges, vec!["SIL 2"]);
    assert!(stage.emphasis);
    assert_eq!(stage.tone, Tone::Warn);
    let arc = g.node("s-inv-inverter-arc").unwrap().mark.clone().expect("arc is marked");
    assert_eq!(arc.badges, vec!["optional"]);
    assert_eq!(arc.value.as_deref(), Some("when Afci"));
    assert_eq!(arc.tone, Tone::Neutral);
    assert!(g.node("s-inv-inverter-board").unwrap().mark.is_none(), "an unmarked part keeps its colours");
}

#[test]
fn group_by_domain_frames_the_parts() {
    let g = graph(&model("groupBy: domain\n"));
    let lanes: Vec<(&str, &str)> = g.nodes.iter().filter(|n| n.kind == NodeKind::Swimlane).map(|n| (n.id.as_str(), n.label.as_str())).collect();
    assert_eq!(lanes, vec![("s-inv-inverter-domain-hardware", "hardware")], "the firmware is inside the board, so no software frame");
    assert_eq!(parent(&g, "s-inv-inverter-board"), Some("s-inv-inverter-domain-hardware"));
    assert_eq!(parent(&g, "s-inv-inverter-stage"), Some("s-inv-inverter-domain-hardware"));
    // Containment wins over the domain frame; the domain becomes a banner.
    assert_eq!(parent(&g, "s-inv-inverter-firmware"), Some("s-inv-inverter-board"));
    let fw = g.node("s-inv-inverter-firmware").unwrap();
    assert!(fw.banners.contains(&"software".to_string()) && fw.mark.is_none());
    parents_precede_children(&g);
}

#[test]
fn unknown_group_by_is_w420() {
    let root = model("groupBy: colour\n");
    let f: Vec<_> = validate(&walk_model(&root).unwrap()).findings.into_iter().filter(|f| f.code == "W420").collect();
    assert_eq!(f.len(), 1, "{f:?}");
    assert!(f[0].message.contains("colour"));
    assert!(graph(&root).nodes.iter().all(|n| n.kind != NodeKind::Swimlane));
}

#[test]
fn edges_cross_nesting_through_ports() {
    let g = graph(&model("depth: 2\n"));
    let e = g.edges.iter().find(|e| e.source == "s-inv-inverter-board-pwm").expect("board.pwm → stage.gate");
    assert_eq!(e.target, "s-inv-inverter-stage-gate");
}

#[test]
fn without_the_new_fields_the_part_tree_is_as_before() {
    // No allocation, no levels, no appliesWhen: same nodes and parents as the one-level generator drew.
    let root = model("");
    std::fs::remove_file(root.join("Inv/FwToMcu.md")).unwrap();
    std::fs::write(root.join("Inv/Arc.md"), "---\ntype: PartDef\nname: Arc\ndomain: hardware\n---\n").unwrap();
    std::fs::write(root.join("Inv/Stage.md"), "---\ntype: PartDef\nname: Stage\ndomain: hardware\nfeatures:\n  - {name: gate, type: Port, direction: in}\n---\n").unwrap();
    let g = graph(&root);
    assert!(g.nodes.iter().all(|n| n.mark.is_none()));
    let blocks: Vec<(&str, Option<&str>)> = g.nodes.iter().filter(|n| n.kind == NodeKind::Block).map(|n| (n.id.as_str(), n.parent.as_deref())).collect();
    assert_eq!(
        blocks,
        vec![
            ("s-inv-inverter-board", Some("s-inv-inverter")),
            ("s-inv-inverter-firmware", Some("s-inv-inverter")),
            ("s-inv-inverter-stage", Some("s-inv-inverter")),
            ("s-inv-inverter-arc", Some("s-inv-inverter")),
        ]
    );
    let ports: Vec<(&str, Option<&str>)> = g.nodes.iter().filter(|n| n.kind == NodeKind::Port).map(|n| (n.id.as_str(), n.parent.as_deref())).collect();
    assert_eq!(ports, vec![("s-inv-inverter-board-pwm", Some("s-inv-inverter-board")), ("s-inv-inverter-stage-gate", Some("s-inv-inverter-stage"))]);
    let edges: Vec<(&str, &str)> = g.edges.iter().map(|e| (e.source.as_str(), e.target.as_str())).collect();
    assert_eq!(edges, vec![("s-inv-inverter-board-pwm", "s-inv-inverter-stage-gate")]);
}

// ── review findings (PR #354) ───────────────────────────────────────────

fn hits(root: &Path, code: &str) -> Vec<syscribe_model::validator::Finding> {
    validate(&walk_model(root).unwrap()).findings.into_iter().filter(|f| f.code == code).collect()
}

/// A bare root with the given files.
fn bare(files: &[(&str, &str)]) -> PathBuf {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "P/_index.md", "---\ntype: Package\nname: P\n---\n");
    write(&root, "Diagrams/_index.md", "---\ntype: Package\nname: Diagrams\n---\n");
    for (rel, content) in files {
        write(&root, rel, content);
    }
    root
}

fn ibd(subject: &str, extra: &str) -> String {
    format!("---\ntype: Diagram\nname: Ibd\ndiagramKind: IBD\nsubject: {subject}\n{extra}---\n\nIBD.\n")
}

#[test]
fn a_definition_that_contains_itself_is_opened_once_and_depth_is_capped() {
    let root = bare(&[
        ("P/Top.md", "---\ntype: PartDef\nname: Top\nfeatures:\n  - {name: n, typedBy: P::Node}\n---\n"),
        ("P/Node.md", "---\ntype: PartDef\nname: Node\nfeatures:\n  - {name: left, typedBy: P::Node}\n  - {name: right, typedBy: P::Node}\n---\n"),
        ("Diagrams/Ibd.md", &ibd("P::Top", "depth: 2000\n")),
    ]);
    let g = graph(&root);
    let blocks: Vec<&str> = g.nodes.iter().filter(|n| n.kind == NodeKind::Block).map(|n| n.id.as_str()).collect();
    assert_eq!(blocks, vec!["s-p-top-n", "s-p-top-n-left", "s-p-top-n-right"], "Node is not opened inside Node");
    assert!(hits(&root, "W420").is_empty());
    assert_eq!(syscribe_model::vis::derive::ibd::MAX_DEPTH, 16);
}

#[test]
fn each_package_goes_into_the_nearest_host_and_packages_pair_with_hosts() {
    let common = [
        ("P/Mcu.md", "---\ntype: PartDef\nname: Mcu\ndomain: hardware\n---\n"),
        ("P/Fw.md", "---\ntype: PartDef\nname: Fw\ndomain: software\nisDeploymentPackage: true\n---\n"),
        ("P/A1.md", "---\ntype: Allocation\nname: A1\nallocatedFrom: [P::Fw]\nallocatedTo: [P::Mcu]\n---\n"),
        ("P/Board.md", "---\ntype: PartDef\nname: Board\ndomain: hardware\nfeatures:\n  - {name: mcu, typedBy: P::Mcu}\n  - {name: fw, typedBy: P::Fw}\n---\n"),
    ];
    // two boards, each with its MCU and firmware: each firmware in its own board's MCU
    let mut files = common.to_vec();
    files.push(("P/Top.md", "---\ntype: PartDef\nname: Top\nfeatures:\n  - {name: b1, typedBy: P::Board}\n  - {name: b2, typedBy: P::Board}\n---\n"));
    let ibd2 = ibd("P::Top", "depth: 2\n");
    files.push(("Diagrams/Ibd.md", &ibd2));
    let g = graph(&bare(&files));
    assert_eq!(parent(&g, "s-p-top-b1-fw"), Some("s-p-top-b1-mcu"));
    assert_eq!(parent(&g, "s-p-top-b2-fw"), Some("s-p-top-b2-mcu"));
    // two firmware images and two MCUs side by side: one each
    let mut files = common.to_vec();
    files.push(("P/Duo.md", "---\ntype: PartDef\nname: Duo\nfeatures:\n  - {name: m1, typedBy: P::Mcu}\n  - {name: m2, typedBy: P::Mcu}\n  - {name: fwA, typedBy: P::Fw}\n  - {name: fwB, typedBy: P::Fw}\n---\n"));
    let ibd1 = ibd("P::Duo", "");
    files.push(("Diagrams/Ibd.md", &ibd1));
    let g = graph(&bare(&files));
    assert_eq!(parent(&g, "s-p-duo-fwa"), Some("s-p-duo-m1"));
    assert_eq!(parent(&g, "s-p-duo-fwb"), Some("s-p-duo-m2"));
}

#[test]
fn members_whose_ids_collide_are_all_drawn() {
    let root = bare(&[
        ("P/Hw.md", "---\ntype: PartDef\nname: Hw\ndomain: hardware\n---\n"),
        // a port and a part both named `comms`, and a part named like a domain frame
        ("P/Board.md", "---\ntype: PartDef\nname: Board\ndomain: hardware\nfeatures:\n  - {name: comms, type: Port, direction: inout}\n  - {name: comms, typedBy: P::Hw}\n---\n"),
        ("P/Top.md", "---\ntype: PartDef\nname: Top\nfeatures:\n  - {name: board, typedBy: P::Board}\n  - {name: domain_hardware, typedBy: P::Hw}\n---\n"),
        ("Diagrams/Ibd.md", &ibd("P::Top", "depth: 2\ngroupBy: domain\n")),
    ]);
    let g = graph(&root);
    assert_eq!(g.node("s-p-top-board-comms").unwrap().kind, NodeKind::Port);
    assert_eq!(g.node("s-p-top-board-comms-2").unwrap().label, "comms : Hw");
    let mut ids: Vec<&str> = g.nodes.iter().map(|n| n.id.as_str()).collect();
    let n = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), n, "node ids are unique");
    assert!(g.nodes.iter().any(|n| n.kind == NodeKind::Swimlane && n.label == "hardware"));
    parents_precede_children(&g);
}

#[test]
fn child_part_files_are_nested_at_depth_two() {
    let root = bare(&[
        ("P/Mcu.md", "---\ntype: PartDef\nname: Mcu\n---\n"),
        ("P/Board.md", "---\ntype: PartDef\nname: Board\n---\n"),
        ("P/Board/mcuA.md", "---\ntype: Part\nname: mcuA\ntypedBy: P::Mcu\n---\n"),
        ("P/Top.md", "---\ntype: PartDef\nname: Top\nfeatures:\n  - {name: b1, typedBy: P::Board}\n---\n"),
        ("Diagrams/Ibd.md", &ibd("P::Top", "depth: 2\n")),
    ]);
    assert_eq!(parent(&graph(&root), "s-p-top-b1-mcua"), Some("s-p-top-b1"));
}

#[test]
fn asil_badges_keep_the_decomposition_and_drop_a_repeated_prefix() {
    let badge = |asil: &str| {
        let root = bare(&[
            ("P/S.md", &format!("---\ntype: PartDef\nname: S\nasilLevel: \"{asil}\"\n---\n")),
            ("P/Top.md", "---\ntype: PartDef\nname: Top\nfeatures:\n  - {name: s, typedBy: P::S}\n---\n"),
            ("Diagrams/Ibd.md", &ibd("P::Top", "")),
        ]);
        graph(&root).node("s-p-top-s").unwrap().mark.clone().map(|m| m.badges)
    };
    assert_eq!(badge("B(D)"), Some(vec!["ASIL B(D)".to_string()]));
    assert_eq!(badge("ASIL C"), Some(vec!["ASIL C".to_string()]));
    assert_eq!(badge("QM"), None);
}

#[test]
fn group_by_outside_a_derived_ibd_is_w420() {
    let root = bare(&[
        ("P/Top.md", "---\ntype: PartDef\nname: Top\ngroupBy: domain\n---\n"),
        ("Diagrams/Bdd.md", "---\ntype: Diagram\nname: Bdd\ndiagramKind: BDD\nsubject: P::Top\ngroupBy: domain\n---\n\nBDD.\n"),
        ("Diagrams/Man.md", "---\ntype: Diagram\nname: Man\ndiagramKind: IBD\ngroupBy: domain\nshapes:\n  - {ref: P::Top}\n---\n\nManifest.\n"),
    ]);
    let f = hits(&root, "W420");
    assert_eq!(f.len(), 3, "{f:?}");
}

#[test]
fn plantuml_and_dot_draw_marks_and_containment() {
    let root = model("depth: 2\ngroupBy: domain\n");
    let elements = walk_model(&root).unwrap();
    let d = elements.iter().find(|e| e.qualified_name == "Diagrams::Ibd").unwrap();
    let puml = syscribe_model::plantuml::render_plantuml(d, &elements, None).unwrap();
    assert!(puml.contains("<<SIL 2>>") && puml.contains("line.bold"), "{puml}");
    assert!(puml.contains("<<optional>>") && puml.contains("line.dashed"), "{puml}");
    let dot = syscribe_model::vis::render_dot(&graph(&root));
    assert!(dot.contains("subgraph cluster_s_inv_inverter_domain_hardware"), "{dot}");
    assert!(dot.contains("subgraph cluster_s_inv_inverter_board_mcu"), "the MCU holding the firmware is a cluster: {dot}");
}

#[test]
fn a_projected_variant_badges_nothing_optional() {
    let root = model("");
    let mut elements = walk_model(&root).unwrap();
    syscribe_model::vis::settle_variant(&mut elements);
    let resolver = Resolver::new(&elements);
    let d = elements.iter().find(|e| e.qualified_name == "Diagrams::Ibd").unwrap();
    let g = build_graph(d, &elements, &resolver).unwrap().0;
    assert!(g.node("s-inv-inverter-arc").unwrap().mark.is_none());
    assert!(g.node("s-inv-inverter-stage").unwrap().mark.is_some(), "the safety mark stays");
}
