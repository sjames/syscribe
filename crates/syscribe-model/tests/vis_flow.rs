//! `REQ-TRS-VIS-028` (GH #335): flow kinds on connections and ports. A `flowKind:` on a `connections:` entry, a
//! port feature or a `PortDef` styles the edge in every writer; the `[diagram]` table of `.syscribe.toml`
//! restyles a kind and places ports by what they carry; an unknown kind, side or table entry is `W419`. A model
//! that names no flow kind draws exactly as before.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::config::ValidateConfig;
use syscribe_model::resolver::Resolver;
use syscribe_model::validator::{validate_with_config, Finding};
use syscribe_model::vis::flow::{DiagramConfig, PortSideRule};
use syscribe_model::vis::ir::FlowKind;
use syscribe_model::vis::{build_graph_in, render_dot, render_mermaid, render_svg, DiagramGraph, NodeKind, Side};
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-vis-flow-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// An inverter: a DC bus feeding a bridge (power), a controller commanding it (command) and a sensor reporting to
/// the controller (signal). `flowed` sets the kinds; otherwise the same model has none.
fn model(flowed: bool) -> PathBuf {
    let root = tempdir();
    let fk = |k: &str| if flowed { format!("flowKind: {k}\n") } else { String::new() };
    let efk = |k: &str| if flowed { format!("    flowKind: {k}\n") } else { String::new() };
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "Inv/_index.md", "---\ntype: Package\nname: Inv\n---\n");
    write(&root, "Inv/DcPort.md", &format!("---\ntype: PortDef\nname: DcPort\n{}---\n", fk("power")));
    write(&root, "Inv/CmdPort.md", "---\ntype: PortDef\nname: CmdPort\n---\n");
    write(&root, "Inv/SigPort.md", "---\ntype: PortDef\nname: SigPort\n---\n");
    write(
        &root,
        "Inv/Bus.md",
        "---\ntype: PartDef\nname: Bus\nfeatures:\n  - name: dcOut\n    typedBy: Inv::DcPort\n    direction: out\n---\n",
    );
    write(
        &root,
        "Inv/Bridge.md",
        &format!(
            "---\ntype: PartDef\nname: Bridge\nfeatures:\n  - name: dcIn\n    typedBy: Inv::DcPort\n    direction: in\n  - name: gate\n    typedBy: Inv::CmdPort\n    direction: in\n{}---\n",
            if flowed { "    flowKind: command\n" } else { "" }
        ),
    );
    write(
        &root,
        "Inv/Ctl.md",
        "---\ntype: PartDef\nname: Ctl\nfeatures:\n  - name: gateOut\n    typedBy: Inv::CmdPort\n    direction: out\n  - name: senseIn\n    typedBy: Inv::SigPort\n    direction: in\n---\n",
    );
    write(&root, "Inv/Sensor.md", "---\ntype: PartDef\nname: Sensor\nfeatures:\n  - name: out\n    typedBy: Inv::SigPort\n    direction: out\n---\n");
    write(
        &root,
        "Inv/Inverter.md",
        &format!(
            "---\ntype: PartDef\nname: Inverter\nfeatures:\n  - name: bus\n    typedBy: Inv::Bus\n  - name: bridge\n    typedBy: Inv::Bridge\n  - name: ctl\n    typedBy: Inv::Ctl\n  - name: sensor\n    typedBy: Inv::Sensor\nconnections:\n  - from: bus.dcOut\n    to: bridge.dcIn\n  - from: ctl.gateOut\n    to: bridge.gate\n  - from: sensor.out\n    to: ctl.senseIn\n{}---\n",
            efk("signal")
        ),
    );
    write(&root, "Diagrams/_index.md", "---\ntype: Package\nname: Diagrams\n---\n");
    write(&root, "Diagrams/InvIBD.md", "---\ntype: Diagram\nname: InvIBD\ndiagramKind: IBD\nsubject: Inv::Inverter\n---\n\nIBD.\n");
    root
}

fn graph(root: &Path) -> DiagramGraph {
    let elements = walk_model(root).unwrap();
    let resolver = Resolver::new(&elements);
    let d = elements.iter().find(|e| e.qualified_name == "Diagrams::InvIBD").unwrap();
    build_graph_in(d, &elements, &resolver, root).unwrap().0
}

fn findings(root: &Path) -> Vec<Finding> {
    let elements = walk_model(root).unwrap();
    validate_with_config(&elements, &ValidateConfig::with_model_root(root)).findings.into_iter().filter(|f| f.code == "W419").collect()
}

fn no_links(_: &str) -> Option<String> {
    None
}

fn edge_flow(g: &DiagramGraph, from_port: &str) -> Option<FlowKind> {
    g.edges.iter().find(|e| e.source.ends_with(from_port) || e.target.ends_with(from_port)).and_then(|e| e.flow)
}

fn port<'a>(g: &'a DiagramGraph, suffix: &str) -> &'a syscribe_model::vis::Node {
    g.nodes.iter().find(|n| n.kind == NodeKind::Port && n.id.ends_with(suffix)).unwrap_or_else(|| panic!("port {suffix}: {:?}", g.nodes.iter().map(|n| &n.id).collect::<Vec<_>>()))
}

#[test]
fn flow_comes_from_the_entry_else_the_port_else_its_port_def() {
    let g = graph(&model(true));
    // PortDef `flowKind: power` reaches the bus → bridge edge.
    assert_eq!(edge_flow(&g, "bus-dcout"), Some(FlowKind::Power));
    // The bridge's `gate` feature says `command`.
    assert_eq!(edge_flow(&g, "ctl-gateout"), Some(FlowKind::Command));
    // The last entry's own `flowKind: signal`.
    assert_eq!(edge_flow(&g, "sensor-out"), Some(FlowKind::Signal));
    assert_eq!(port(&g, "bridge-gate").flow, Some(FlowKind::Command));
}

#[test]
fn a_model_without_flow_kinds_draws_as_before() {
    let g = graph(&model(false));
    assert!(g.edges.iter().all(|e| e.flow.is_none()));
    assert!(g.nodes.iter().all(|n| n.flow.is_none()));
    let svg = render_svg(&g, &no_links).unwrap();
    assert!(!svg.contains("flow-"), "no flow class without flow kinds");
    let json = serde_json::to_string(&g).unwrap();
    assert!(!json.contains("\"flow\""), "the IR serializes as before");
    let mmd = render_mermaid(&g, &no_links).unwrap();
    assert!(!mmd.contains("linkStyle") && !mmd.contains("%% flow"));
}

#[test]
fn flow_kinds_style_every_writer() {
    let g = graph(&model(true));
    let svg = render_svg(&g, &no_links).unwrap();
    assert!(svg.contains("flow-power") && svg.contains("#b35900"), "power stroke in SVG");
    assert!(svg.contains("flow-command") && svg.contains("6,3"), "command dash in SVG");
    let mmd = render_mermaid(&g, &no_links).unwrap();
    assert!(mmd.contains("%% flow: power"), "{mmd}");
    assert!(mmd.contains("linkStyle") && mmd.contains("stroke:#b35900"), "{mmd}");
    let dot = render_dot(&g);
    assert!(dot.contains("class=\"flow-power\"") && dot.contains("#b35900"), "{dot}");
}

#[test]
fn plantuml_carries_the_kind_as_a_stereotype() {
    let root = model(true);
    let elements = walk_model(&root).unwrap();
    let d = elements.iter().find(|e| e.qualified_name == "Diagrams::InvIBD").unwrap();
    let puml = syscribe_model::plantuml::render_plantuml_in(d, &elements, None, &root).expect("PlantUML for a derived IBD");
    assert!(puml.contains("<<power>>") && puml.contains("#b35900"), "{puml}");
    assert!(puml.contains("dashed"), "command is dashed: {puml}");
}

#[test]
fn the_diagram_table_restyles_a_kind() {
    let root = model(true);
    write(&root, ".syscribe.toml", "[diagram.styles.power]\nstroke = \"#ff0000\"\nwidth = 4\n");
    let g = graph(&root);
    let svg = render_svg(&g, &no_links).unwrap();
    assert!(svg.contains("#ff0000") && !svg.contains("#b35900"), "override replaces the built-in stroke");
    assert!(findings(&root).is_empty());
}

#[test]
fn port_side_rule_places_ports_by_flow_and_a_port_side_wins() {
    let root = model(true);
    // Without a rule every side is left to the direction rule, as today.
    assert!(graph(&root).nodes.iter().filter(|n| n.kind == NodeKind::Port).all(|n| n.side.is_none()));

    write(&root, ".syscribe.toml", "[diagram.portSides]\npower = \"direction\"\nsignal = \"south\"\ncommand = \"north\"\n");
    let g = graph(&root);
    assert_eq!(port(&g, "bus-dcout").side, None, "power keeps the direction rule");
    assert_eq!(port(&g, "sensor-out").side, Some(Side::South), "signal goes south through its edge");
    assert_eq!(port(&g, "ctl-sensein").side, Some(Side::South));
    assert_eq!(port(&g, "bridge-gate").side, Some(Side::North));

    // A port's own `side:` wins over the rule.
    let inv = std::fs::read_to_string(root.join("Inv/Sensor.md")).unwrap();
    write(&root, "Inv/Sensor.md", &inv.replace("direction: out\n", "direction: out\n    side: east\n"));
    assert_eq!(port(&graph(&root), "sensor-out").side, Some(Side::East));
}

#[test]
fn config_parses_styles_and_rules() {
    let c = DiagramConfig::from_toml_str("[diagram.styles.data]\ncolor = \"#00f\"\ndash = \"\"\n[diagram.port_sides]\ndata = \"west\"\n");
    assert!(c.problems.is_empty(), "{:?}", c.problems);
    assert_eq!(c.styles[&FlowKind::Data].stroke.as_deref(), Some("#00f"));
    assert_eq!(c.port_sides[&FlowKind::Data], PortSideRule::Fixed(Side::West));
}

#[test]
fn unknown_kinds_sides_and_table_entries_are_w419() {
    let root = model(true);
    write(&root, ".syscribe.toml", "[diagram]\ncolours = 1\n[diagram.styles.hydraulic]\nstroke = \"#000\"\n[diagram.portSides]\npower = \"up\"\n");
    let inv = std::fs::read_to_string(root.join("Inv/Inverter.md")).unwrap();
    write(&root, "Inv/Inverter.md", &inv.replace("flowKind: signal", "flowKind: pneumatic"));
    let ctl = std::fs::read_to_string(root.join("Inv/Ctl.md")).unwrap();
    write(&root, "Inv/Ctl.md", &ctl.replace("    direction: in\n", "    direction: in\n    side: top\n"));
    write(&root, "Inv/SigPort.md", "---\ntype: PortDef\nname: SigPort\nflowKind: analog\n---\n");

    let f = findings(&root);
    let msgs: Vec<&str> = f.iter().map(|f| f.message.as_str()).collect();
    for needle in ["colours", "hydraulic", "'up'", "pneumatic", "side 'top'", "analog"] {
        assert!(msgs.iter().any(|m| m.contains(needle)), "no W419 naming {needle}: {msgs:#?}");
    }
    assert_eq!(f.len(), 6, "{msgs:#?}");
    // The bad entry is ignored: the edge falls back to the port's flow (none here), so it draws as signal would.
    let g = graph(&root);
    assert_eq!(edge_flow(&g, "sensor-out"), None);
}

#[test]
fn plantuml_thickness_is_an_integer_and_signal_keeps_the_plain_arrow() {
    let root = model(true);
    let elements = walk_model(&root).unwrap();
    let d = elements.iter().find(|e| e.qualified_name == "Diagrams::InvIBD").unwrap();
    let puml = syscribe_model::plantuml::render_plantuml_in(d, &elements, None, &root).unwrap();
    for line in puml.lines().filter(|l| l.contains("thickness=")) {
        let t = line.split("thickness=").nth(1).unwrap();
        let digits: String = t.chars().take_while(|c| c.is_ascii_digit()).collect();
        assert!(!digits.is_empty() && !t[digits.len()..].starts_with('.'), "PlantUML takes an integer thickness: {line}");
        assert!(line.contains("-[#"), "a PlantUML colour starts with #: {line}");
    }
    let signal = puml.lines().find(|l| l.contains("<<signal>>")).expect("the signal edge");
    assert!(signal.contains(" --> "), "signal adds no style: {signal}");
}

#[test]
fn mermaid_styles_only_edges_whose_kind_changes_their_look_at_their_own_index() {
    let g = graph(&model(true));
    let mmd = render_mermaid(&g, &no_links).unwrap();
    let styled: Vec<usize> = mmd.lines().filter_map(|l| l.trim().strip_prefix("linkStyle ")).map(|r| r.split(' ').next().unwrap().parse().unwrap()).collect();
    let expected: Vec<usize> = g.edges.iter().enumerate().filter(|(_, e)| matches!(e.flow, Some(FlowKind::Power) | Some(FlowKind::Command))).map(|(i, _)| i).collect();
    assert_eq!(styled, expected, "{mmd}");
}

#[test]
fn dot_draws_data_dotted_and_command_dashed() {
    let root = model(true);
    let inv = std::fs::read_to_string(root.join("Inv/Inverter.md")).unwrap();
    write(&root, "Inv/Inverter.md", &inv.replace("flowKind: signal", "flowKind: data"));
    let dot = render_dot(&graph(&root));
    let line = |k: &str| dot.lines().find(|l| l.contains(&format!("flow-{k}"))).unwrap().to_string();
    assert!(line("data").contains("style=dotted"), "{dot}");
    assert!(line("command").contains("style=dashed"), "{dot}");
}

#[test]
fn a_binding_carries_no_flow_kind() {
    let root = model(true);
    let inv = std::fs::read_to_string(root.join("Inv/Inverter.md")).unwrap();
    write(&root, "Inv/Inverter.md", &inv.replace("connections:\n  - from: bus.dcOut\n    to: bridge.dcIn\n", "bindingConnections:\n  - left: bus.dcOut\n    right: bridge.dcIn\nconnections:\n"));
    let g = graph(&root);
    let b = g.edges.iter().find(|e| e.source.ends_with("bus-dcout")).expect("the binding");
    assert_eq!(b.flow, None);
}

#[test]
fn a_port_def_inherits_its_supertypes_flow_kind() {
    let root = model(true);
    write(&root, "Inv/HvDcPort.md", "---\ntype: PortDef\nname: HvDcPort\nsupertype: Inv::DcPort\n---\n");
    let bus = std::fs::read_to_string(root.join("Inv/Bus.md")).unwrap();
    write(&root, "Inv/Bus.md", &bus.replace("Inv::DcPort", "Inv::HvDcPort"));
    let g = graph(&root);
    assert_eq!(port(&g, "bus-dcout").flow, Some(FlowKind::Power));
}

#[test]
fn a_manifest_diagram_takes_the_flow_kind_of_its_ports() {
    let root = model(true);
    write(&root, "Inv/BusOut.md", "---\ntype: Port\nname: BusOut\ntypedBy: Inv::DcPort\ndirection: out\n---\n");
    write(
        &root,
        "Diagrams/Hand.md",
        "---\ntype: Diagram\nname: Hand\ndiagramKind: IBD\nshapes:\n  bus: {ref: Inv::Bus, kind: block}\n  out: {ref: Inv::BusOut, kind: port, parent: bus}\n  bridge: {ref: Inv::Bridge, kind: block}\nedges:\n  e1: {source: out, target: bridge, kind: connection}\n---\n\nHand.\n",
    );
    let elements = walk_model(&root).unwrap();
    let resolver = Resolver::new(&elements);
    let d = elements.iter().find(|e| e.qualified_name == "Diagrams::Hand").unwrap();
    let g = build_graph_in(d, &elements, &resolver, &root).unwrap().0;
    assert_eq!(g.node("out").unwrap().flow, Some(FlowKind::Power));
    assert_eq!(g.edges[0].flow, Some(FlowKind::Power));
}

#[test]
fn w419_covers_every_place_a_value_is_read_or_ignored() {
    let root = model(false);
    write(&root, ".syscribe.toml", "[diagram.styles.power]\nstroke = \"rgb(1, 2, 3)\"\ndash = \"a,b\"\n");
    write(&root, "Inv/Bus.md", "---\ntype: PartDef\nname: Bus\nflowKind: power\nfeatures:\n  - name: mass\n    typedBy: ScalarValues::Real\n    flowKind: power\n    side: north\n  - name: dcOut\n    typedBy: Inv::DcPort\n    direction: out\n---\n");
    let inv = std::fs::read_to_string(root.join("Inv/Inverter.md")).unwrap();
    write(
        &root,
        "Inv/Inverter.md",
        &inv.replace("connections:\n", "flowConnections:\n  - {from: ctl.gateOut, to: bridge.gate, flowKind: hydraulic}\nbindingConnections:\n  - {left: bus.dcOut, right: bridge.dcIn, flowKind: power}\nconnections:\n").replacen("    to: bridge.dcIn\n", "    to: bridge.dcIn\n    side: west\n", 1),
    );
    write(&root, "Diagrams/Hand.md", "---\ntype: Diagram\nname: Hand\ndiagramKind: IBD\nshapes:\n  a: {ref: Inv::Bus, kind: block}\n  b: {ref: Inv::Bridge, kind: block}\nedges:\n  e1: {source: a, target: b, kind: connection, flowKind: steam}\n---\n\nHand.\n");
    let f = findings(&root);
    let msgs: Vec<&str> = f.iter().map(|f| f.message.as_str()).collect();
    for needle in [
        "'rgb(1, 2, 3)' is not a colour",
        "dash 'a,b'",
        "ignored on a PartDef",
        "'mass': flowKind is read only on a port",
        "'mass': side is read only on a port",
        "flowKind 'hydraulic'",
        "flowKind is read only on connections and flowConnections",
        "side is read only on a port feature",
        "edges entry 'e1': flowKind 'steam'",
    ] {
        assert!(msgs.iter().any(|m| m.contains(needle)), "no W419 with {needle:?}: {msgs:#?}");
    }
    assert_eq!(f.len(), 9, "{msgs:#?}");
}
