//! `REQ-TRS-VIS-009`/`-010`: the Mermaid and static-SVG writers are pure
//! functions of the Diagram IR. Their output for the derived BDD/IBD of the
//! `vis_derive` fixture model is pinned by golden snapshots under
//! `tests/vis_snapshots/writers/` — refresh with
//! `SYSCRIBE_UPDATE_SNAPSHOTS=1 cargo test -p syscribe-model --test vis_writers`
//! and review the diff. The unit assertions cover the contract the snapshots
//! cannot name: a `%% ref:` per Mermaid node, `sysml:ref` per SVG node and
//! `sysml:source`/`sysml:target` per edge, an unpinned graph laid out by the
//! embedded ELK (`REQ-TRS-VIS-016`), and the `REQ-TRS-LINK-002` hyperlink
//! wrapper driven by the `links` closure. The SVG tests size the graph with
//! the approximate metrics so the snapshots do not depend on the fonts
//! installed on the machine running them.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::resolver::Resolver;
use syscribe_model::vis::metrics::ApproxMetrics;
use syscribe_model::vis::svg::render_svg_with;
use syscribe_model::vis::{build_graph, render_mermaid, size_graph, DiagramGraph, NodeKind, SvgError};
use syscribe_model::walker::walk_model;

/// The SVG writer over the font-independent approximate metrics.
fn render_svg(g: &DiagramGraph, links: &dyn Fn(&str) -> Option<String>) -> Result<String, SvgError> {
    render_svg_with(g, &size_graph(g, &ApproxMetrics), links)
}

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-vis-writers-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// The `vis_derive` fixture: a `Sys` package with an abstract base, two
/// definitions with ports, a port definition, a connection definition and a
/// composed `PowerSystem` with inline part usages, a boundary port, a
/// connection and a binding, plus one file-per-usage child part.
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

/// A `layout:` pinning every node of the derived BDD (blocks and their
/// compartments, parent-relative) with measured sizes.
const BDD_LAYOUT: &str = "layout:
  s-sys-base: {x: 300, y: 20, w: 160, h: 50}
  s-sys-engine: {x: 120, y: 160, w: 200, h: 84}
  s-sys-engine-compartment: {x: 0, y: 50, w: 200, h: 34}
  s-sys-motor: {x: 480, y: 160, w: 200, h: 70}
  s-sys-motor-compartment: {x: 0, y: 50, w: 200, h: 20}
  s-sys-powerlink: {x: 300, y: 300, w: 160, h: 50}
  s-sys-powerport: {x: 20, y: 20, w: 160, h: 50}
  s-sys-powersystem: {x: 300, y: 420, w: 200, h: 70}
  s-sys-powersystem-compartment: {x: 0, y: 50, w: 200, h: 20}
";

/// A `layout:` pinning every node of the derived IBD: the boundary without a
/// size (it bounds its children), blocks and ports parent-relative.
const IBD_LAYOUT: &str = "layout:
  s-sys-powersystem: {x: 0, y: 0}
  s-sys-powersystem-mainout: {x: 474, y: 100, w: 12, h: 12}
  s-sys-powersystem-engine: {x: 40, y: 60, w: 160, h: 50}
  s-sys-powersystem-engine-powerout: {x: 154, y: 19, w: 12, h: 12}
  s-sys-powersystem-motor: {x: 300, y: 60, w: 160, h: 50}
  s-sys-powersystem-motor-powerin: {x: -6, y: 19, w: 12, h: 12}
  s-sys-powersystem-aux: {x: 300, y: 160, w: 160, h: 50}
  s-sys-powersystem-aux-powerin: {x: -6, y: 19, w: 12, h: 12}
  e-binding-s-sys-powersystem-motor-powerin-s-sys-powersystem-mainout:
    points: [[380, 130], [500, 130]]
";

fn graph_of(root: &Path, qname: &str) -> DiagramGraph {
    let elements = walk_model(root).unwrap();
    let resolver = Resolver::new(&elements);
    let d = elements.iter().find(|e| e.qualified_name == qname).expect("diagram element");
    build_graph(d, &elements, &resolver).expect("an IR").0
}

fn no_links(_: &str) -> Option<String> {
    None
}

fn snapshot_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/vis_snapshots/writers")
}

fn assert_snapshot(name: &str, actual: &str) {
    let path = snapshot_dir().join(name);
    if std::env::var_os("SYSCRIBE_UPDATE_SNAPSHOTS").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("missing snapshot {}: {e} (run with SYSCRIBE_UPDATE_SNAPSHOTS=1)", path.display()));
    assert_eq!(actual, expected, "golden {name} changed; review and refresh with SYSCRIBE_UPDATE_SNAPSHOTS=1");
}

// ── Mermaid ─────────────────────────────────────────────────────────────────

#[test]
fn mermaid_of_the_derived_bdd_matches_its_snapshot_and_annotates_every_node() {
    let root = fixture_model();
    add_diagram(&root, "SysBDD", "diagramKind: BDD\nsubject: Sys\n");
    let g = graph_of(&root, "Diagrams::SysBDD");
    let m = render_mermaid(&g, &no_links).expect("BDD maps to a classDiagram");
    assert!(m.starts_with("classDiagram\n"));
    // Every drawn node (blocks; compartments fold into their block) has a ref line.
    for n in g.nodes.iter().filter(|n| n.kind == NodeKind::Block) {
        assert!(m.contains(&format!("%% ref: {}\n", n.element_ref)), "missing ref for {}: {m}", n.id);
    }
    assert_eq!(m.matches("%% ref:").count(), g.nodes.iter().filter(|n| n.kind == NodeKind::Block).count());
    assert!(m.contains("<<part def>>"));
    assert!(m.contains("+mass : Real [kg]"), "compartment lines are class members: {m}");
    assert!(m.contains("s_sys_base <|-- s_sys_engine"), "inheritance: {m}");
    assert!(m.contains("s_sys_powersystem *-- s_sys_motor : motor [2]"), "composition with the usage label: {m}");
    assert!(m.contains("s_sys_engine -- s_sys_motor : PowerLink"), "association: {m}");
    assert!(!m.contains("click "));
    assert_snapshot("sys_bdd.mmd", &m);
}

#[test]
fn mermaid_of_the_derived_ibd_matches_its_snapshot_and_nests_subgraphs() {
    let root = fixture_model();
    add_diagram(&root, "PowerIBD", "diagramKind: IBD\nsubject: Sys::PowerSystem\n");
    let g = graph_of(&root, "Diagrams::PowerIBD");
    let m = render_mermaid(&g, &no_links).expect("IBD maps to a flowchart");
    assert!(m.starts_with("flowchart LR\n"));
    for n in &g.nodes {
        assert!(m.contains(&format!("%% ref: {}\n", n.element_ref)), "missing ref for {}: {m}", n.id);
    }
    assert_eq!(m.matches("%% ref:").count(), g.nodes.len());
    assert!(m.contains("subgraph s_sys_powersystem[\"PowerSystem\"]"), "{m}");
    assert!(m.contains("  subgraph s_sys_powersystem_engine[\"engine : Engine\"]"), "a block with ports is a nested subgraph: {m}");
    assert!(m.contains("s_sys_powersystem_engine_powerout((powerOut))"), "ports are small nodes: {m}");
    assert!(m.contains("s_sys_powersystem_engine_powerout ---|PowerLink| s_sys_powersystem_motor_powerin"), "{m}");
    assert!(m.contains("s_sys_powersystem_motor_powerin -.-|=| s_sys_powersystem_mainout"), "{m}");
    assert_snapshot("power_ibd.mmd", &m);
}

#[test]
fn mermaid_click_lines_follow_the_links_closure() {
    let root = fixture_model();
    add_diagram(&root, "SysBDD", "diagramKind: BDD\nsubject: Sys\n");
    let g = graph_of(&root, "Diagrams::SysBDD");
    let links = |q: &str| (q == "Sys::Engine" || q == "Sys::Motor").then(|| format!("https://host.test/{}.md", q.replace("::", "/")));
    let m = render_mermaid(&g, &links).unwrap();
    assert!(m.contains("\n  click s_sys_engine href \"https://host.test/Sys/Engine.md\" _blank\n"), "{m}");
    assert!(m.contains("\n  click s_sys_motor href \"https://host.test/Sys/Motor.md\" _blank\n"), "{m}");
    assert_eq!(m.matches("click ").count(), 2, "only linked nodes get a click line");
    assert!(!render_mermaid(&g, &no_links).unwrap().contains("click "));
}

// ── SVG ─────────────────────────────────────────────────────────────────────

#[test]
fn svg_of_the_pinned_derived_bdd_matches_its_snapshot_with_sysml_attributes() {
    let root = fixture_model();
    add_diagram(&root, "SysBDD", &format!("diagramKind: BDD\nsubject: Sys\n{BDD_LAYOUT}"));
    let g = graph_of(&root, "Diagrams::SysBDD");
    assert!(g.is_fully_pinned());
    let s = render_svg(&g, &no_links).expect("a fully pinned graph draws");
    assert!(s.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:sysml=\"urn:syscribe:1.0\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" viewBox=\"0 0 "), "{s}");
    for n in &g.nodes {
        assert!(s.contains(&format!("<g id=\"{}\" class=\"{}", n.id, n.kind.as_str())), "node group for {}: {s}", n.id);
        assert!(s.contains(&format!("sysml:ref=\"{}\"", n.element_ref)), "sysml:ref for {}", n.id);
    }
    for e in &g.edges {
        assert!(s.contains(&format!("<path id=\"{}\" class=\"edge {}\"", e.id, e.kind.as_str())), "edge {}: {s}", e.id);
        assert!(s.contains(&format!("sysml:source=\"{}\" sysml:target=\"{}\"", e.source, e.target)), "endpoints of {}", e.id);
    }
    assert!(s.contains("<marker id=\"arrow-inherit\"") && s.contains("<marker id=\"arrow-composition\""), "{s}");
    assert!(s.contains("font-style=\"italic\""), "the abstract Base is italic");
    assert!(s.contains("«part def»") && s.contains("«connection def»"));
    assert!(!s.contains("<a "), "no [links], no wrapper");
    assert_snapshot("sys_bdd.svg", &s);
}

#[test]
fn svg_of_the_pinned_derived_ibd_matches_its_snapshot_and_bounds_the_boundary() {
    let root = fixture_model();
    add_diagram(&root, "PowerIBD", &format!("diagramKind: IBD\nsubject: Sys::PowerSystem\n{IBD_LAYOUT}"));
    let g = graph_of(&root, "Diagrams::PowerIBD");
    let s = render_svg(&g, &no_links).expect("a fully pinned graph draws");
    for n in &g.nodes {
        assert!(s.contains(&format!("sysml:ref=\"{}\"", n.element_ref)), "sysml:ref for {}", n.id);
    }
    assert_eq!(s.matches("<g id=\"").count(), g.nodes.len());
    assert_eq!(s.matches("<path id=\"e-").count(), g.edges.len());
    // The unsized boundary bounds its children: 474+12+16 wide (the boundary
    // port sticks out furthest), 160+50+16 tall, at the 20 margin.
    assert!(s.contains("<g id=\"s-sys-powersystem\" class=\"boundary PartDef\" sysml:ref=\"Sys::PowerSystem\">\n    <rect x=\"20\" y=\"20\" width=\"502\" height=\"226\" rx=\"8\""), "{s}");
    // Ports: class carries the direction; fill follows the port style (out dark, in white).
    assert!(s.contains("class=\"port Port out\"") && s.contains("class=\"port Port in\""), "{s}");
    assert!(s.contains("sysml:source=\"s-sys-powersystem-engine-powerout\" sysml:target=\"s-sys-powersystem-motor-powerin\""), "{s}");
    // The binding follows its pinned waypoints (shifted by the margin) and shows `=`.
    assert!(s.contains("L 400,150 L 520,150"), "waypoints: {s}");
    assert!(s.contains(">=</text>"), "{s}");
    assert_snapshot("power_ibd.svg", &s);
}

#[test]
fn svg_of_the_unpinned_derived_ibd_is_laid_out_by_elk_and_matches_its_snapshot() {
    // REQ-TRS-VIS-016: no pins at all — the embedded ELK lays the graph out
    // and the writer draws its rectangles, routed edges and label positions.
    let root = fixture_model();
    add_diagram(&root, "Bare", "diagramKind: IBD\nsubject: Sys::PowerSystem\n");
    let g = graph_of(&root, "Diagrams::Bare");
    assert!(!g.is_fully_pinned() && g.pinned_ids().is_empty());
    let s = render_svg(&g, &no_links).expect("laid out by the embedded ELK");
    for n in &g.nodes {
        assert!(s.contains(&format!("<g id=\"{}\" class=\"{}", n.id, n.kind.as_str())), "node group for {}: {s}", n.id);
        assert!(s.contains(&format!("sysml:ref=\"{}\"", n.element_ref)), "sysml:ref for {}", n.id);
    }
    assert_eq!(s.matches("<path id=\"e-").count(), g.edges.len(), "every edge routed: {s}");
    assert!(s.contains(">PowerLink</text>") && s.contains(">=</text>"), "edge labels placed by ELK: {s}");
    assert_snapshot("power_ibd_unpinned.svg", &s);

    // Everything but `aux` pinned: still drawn, around the pins.
    let partial = IBD_LAYOUT.lines().filter(|l| !l.contains("s-sys-powersystem-aux:")).collect::<Vec<_>>().join("\n") + "\n";
    add_diagram(&root, "Partial", &format!("diagramKind: IBD\nsubject: Sys::PowerSystem\n{partial}"));
    let g = graph_of(&root, "Diagrams::Partial");
    assert!(!g.is_fully_pinned());
    assert_eq!(g.nodes.iter().filter(|n| n.pin.is_none()).map(|n| n.id.as_str()).collect::<Vec<_>>(), vec!["s-sys-powersystem-aux"]);
    let s = render_svg(&g, &no_links).expect("a partially pinned graph draws");
    assert_eq!(s.matches("<g id=\"").count(), g.nodes.len());

    // Only a graph with nothing to draw is refused.
    add_diagram(&root, "Empty", "diagramKind: IBD\nsubject: Sys\n");
    assert!(matches!(render_svg(&graph_of(&root, "Diagrams::Empty"), &no_links), Err(SvgError::Empty)));
}

#[test]
fn svg_wraps_linked_nodes_in_the_req_trs_link_002_anchor_and_nothing_else() {
    let root = fixture_model();
    add_diagram(&root, "SysBDD", &format!("diagramKind: BDD\nsubject: Sys\n{BDD_LAYOUT}"));
    let g = graph_of(&root, "Diagrams::SysBDD");
    let links = |q: &str| (q == "Sys::Engine").then(|| "https://host.test/Sys/Engine.md?a=1&b=2".to_string());
    let s = render_svg(&g, &links).unwrap();
    assert!(
        s.contains("  <a xlink:href=\"https://host.test/Sys/Engine.md?a=1&amp;b=2\" href=\"https://host.test/Sys/Engine.md?a=1&amp;b=2\" target=\"_blank\" rel=\"noopener\">\n  <g id=\"s-sys-engine\""),
        "{s}"
    );
    assert_eq!(s.matches("<a ").count(), 1, "exactly the linked node is wrapped");
    assert_eq!(s.matches("</a>").count(), 1);
    assert!(!render_svg(&g, &no_links).unwrap().contains("<a "), "no URL, no wrapper");
}
