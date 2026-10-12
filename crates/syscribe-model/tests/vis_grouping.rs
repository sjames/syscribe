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
}
