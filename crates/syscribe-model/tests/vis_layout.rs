//! `REQ-TRS-VIS-016`/`-017`: the embedded ELK (`vis::layout`) and the
//! Rust-owned sizes (`vis::size`). Through the real walker on the
//! `vis_derive` fixture model:
//!
//! - the derived IBD and BDD lay out with no sibling overlap, every port
//!   centre on its parent's border, every edge routed and every label
//!   inside its node;
//! - determinism against Node: the ELK input `vis::layout::elk_input` builds
//!   for the IBD fixture is pinned under `tests/vis_snapshots/elk/
//!   power_ibd.input.json`, and `power_ibd.output.json` is what `elkjs`
//!   under Node produced from it (see the comment at
//!   [`elk_under_quickjs_matches_node_for_the_ibd_fixture`] for the command);
//!   running the same input through QuickJS must give identical coordinates;
//! - a 200-node / 300-edge synthetic BDD lays out well within the bound;
//! - pins: a fully pinned graph uses `fixed` and keeps every position; a
//!   partially pinned one runs ELK's interactive strategies with
//!   `elk.position` on the pinned nodes.
//!
//! Sizes use the approximate metrics so nothing here depends on the fonts
//! installed. Refresh the input snapshot with
//! `SYSCRIBE_UPDATE_SNAPSHOTS=1 cargo test -p syscribe-model --test vis_layout`,
//! then regenerate the output with Node and review both diffs.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use serde_json::{Map, Number, Value};
use syscribe_model::resolver::Resolver;
use syscribe_model::vis::ir::{Edge, EdgeKind, Node, NodeKind, Rect};
use syscribe_model::vis::layout::{elk_input, layout, run_elk, Bounds, Layout};
use syscribe_model::vis::metrics::ApproxMetrics;
use syscribe_model::vis::size::{size_graph, Sizes};
use syscribe_model::vis::{build_graph, DiagramGraph, DiagramKind};
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-vis-layout-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// The `vis_derive` fixture model (see `tests/vis_derive.rs`).
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

fn graph_of(root: &Path, qname: &str) -> DiagramGraph {
    let elements = walk_model(root).unwrap();
    let resolver = Resolver::new(&elements);
    let d = elements.iter().find(|e| e.qualified_name == qname).expect("diagram element");
    build_graph(d, &elements, &resolver).expect("an IR").0
}

fn sizes_of(g: &DiagramGraph) -> Sizes {
    size_graph(g, &ApproxMetrics)
}

fn snapshot_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/vis_snapshots/elk")
}

fn overlaps(a: &Bounds, b: &Bounds) -> bool {
    a.x < b.right() && b.x < a.right() && a.y < b.bottom() && b.y < a.bottom()
}

fn inside(inner: &Bounds, outer: &Bounds, tol: f64) -> bool {
    inner.x >= outer.x - tol && inner.y >= outer.y - tol && inner.right() <= outer.right() + tol && inner.bottom() <= outer.bottom() + tol
}

/// The three properties the picture depends on (and the Node-side
/// `npm test` checks the same way): disjoint siblings, ports on borders,
/// labels inside their nodes; plus every edge routed.
fn check_layout(g: &DiagramGraph, sizes: &Sizes, l: &Layout) -> (usize, usize) {
    let is_node = |n: &Node| !matches!(n.kind, NodeKind::Port | NodeKind::Compartment | NodeKind::Label);
    let mut pairs = 0;
    let mut ports = 0;
    let mut by_parent: BTreeMap<Option<&str>, Vec<&Node>> = BTreeMap::new();
    for n in g.nodes.iter().filter(|n| is_node(n)) {
        by_parent.entry(n.parent.as_deref()).or_default().push(n);
    }
    for (parent, kids) in &by_parent {
        for (i, a) in kids.iter().enumerate() {
            for b in &kids[i + 1..] {
                assert!(!overlaps(&l.nodes[&a.id], &l.nodes[&b.id]), "siblings {} and {} overlap under {parent:?}", a.id, b.id);
                pairs += 1;
            }
        }
        if let Some(p) = parent {
            for k in kids {
                assert!(inside(&l.nodes[&k.id], &l.nodes[*p], 0.5), "{} is not inside its parent {p}", k.id);
            }
        }
    }
    for n in g.nodes.iter().filter(|n| n.kind == NodeKind::Port) {
        let p = &l.nodes[&n.id];
        let parent = &l.nodes[n.parent.as_deref().unwrap()];
        let on_vertical = (p.cx() - parent.x).abs() <= 1.0 || (p.cx() - parent.right()).abs() <= 1.0;
        let on_horizontal = (p.cy() - parent.y).abs() <= 1.0 || (p.cy() - parent.bottom()).abs() <= 1.0;
        assert!(on_vertical || on_horizontal, "port {} centre ({}, {}) is not on the border of {:?}", n.id, p.cx(), p.cy(), parent);
        ports += 1;
    }
    for n in g.nodes.iter().filter(|n| is_node(n)) {
        let b = &l.nodes[&n.id];
        for lab in &sizes.node(&n.id).unwrap().labels {
            let lb = l.labels.get(&lab.id).unwrap_or_else(|| panic!("label {} placed", lab.id));
            assert!(inside(lb, b, 0.5), "label {} {lb:?} is outside its node {:?}", lab.id, b);
        }
    }
    for e in &g.edges {
        let r = l.edges.get(&e.id).unwrap_or_else(|| panic!("edge {} routed", e.id));
        assert!(r.routed && r.points.len() >= 2, "edge {} has a route", e.id);
    }
    (pairs, ports)
}

#[test]
fn the_derived_ibd_lays_out_with_ports_on_borders_and_no_overlap() {
    let root = fixture_model();
    add_diagram(&root, "PowerIBD", "diagramKind: IBD\nsubject: Sys::PowerSystem\n");
    let g = graph_of(&root, "Diagrams::PowerIBD");
    let sizes = sizes_of(&g);
    let t = Instant::now();
    let l = layout(&g, &sizes).expect("layout");
    println!("derived IBD ({} nodes, {} edges): {:?}", g.nodes.len(), g.edges.len(), t.elapsed());
    assert_eq!(l.algorithm, "layered");
    let (pairs, ports) = check_layout(&g, &sizes, &l);
    assert_eq!(ports, 4, "mainOut, engine.powerOut, motor.powerIn, aux.powerIn");
    assert!(pairs >= 3, "engine/motor/aux pairs");
    // Hierarchy: the blocks sit inside the boundary, in root coordinates.
    let boundary = l.nodes["s-sys-powersystem"];
    assert!(boundary.x >= 0.0 && boundary.y >= 0.0);
    assert!(l.width >= boundary.right() && l.height >= boundary.bottom());
    // The connection runs from the engine's out port to the motor's in port
    // (ELK's orthogonal route: its ends on the two ports).
    let e = &l.edges["e-connection-s-sys-powersystem-engine-powerout-s-sys-powersystem-motor-powerin"];
    let (src, tgt) = (l.nodes["s-sys-powersystem-engine-powerout"], l.nodes["s-sys-powersystem-motor-powerin"]);
    let first = e.points.first().unwrap();
    let last = e.points.last().unwrap();
    assert!((first.x - src.cx()).abs() <= src.w && (first.y - src.cy()).abs() <= src.h, "starts at the source port: {first:?} vs {src:?}");
    assert!((last.x - tgt.cx()).abs() <= tgt.w && (last.y - tgt.cy()).abs() <= tgt.h, "ends at the target port: {last:?} vs {tgt:?}");
    // The edge label was placed by ELK.
    assert!(l.labels.contains_key("e-connection-s-sys-powersystem-engine-powerout-s-sys-powersystem-motor-powerin-label"));
    // Compartment-free IBD blocks: the carried size is what ELK used.
    for id in ["s-sys-powersystem-engine", "s-sys-powersystem-motor", "s-sys-powersystem-aux"] {
        let s = sizes.size_of(id);
        assert_eq!((l.nodes[id].w, l.nodes[id].h), (s.w, s.h), "{id} keeps its carried size");
    }
}

#[test]
fn the_derived_bdd_lays_out_with_supertypes_above_and_compartments_inside() {
    let root = fixture_model();
    add_diagram(&root, "SysBDD", "diagramKind: BDD\nsubject: Sys\n");
    let g = graph_of(&root, "Diagrams::SysBDD");
    let sizes = sizes_of(&g);
    let t = Instant::now();
    let l = layout(&g, &sizes).expect("layout");
    println!("derived BDD ({} nodes, {} edges): {:?}", g.nodes.len(), g.edges.len(), t.elapsed());
    let (pairs, _) = check_layout(&g, &sizes, &l);
    assert!(pairs >= 10);
    // Inheritance is reversed for layering: Base sits above Engine and Motor,
    // and the route still runs sub → super after the flip back.
    let base = l.nodes["s-sys-base"];
    for sub in ["s-sys-engine", "s-sys-motor"] {
        assert!(l.nodes[sub].y > base.bottom(), "{sub} below Base");
        let e = &l.edges[&format!("e-inheritance-{sub}-s-sys-base")];
        assert!(e.points.first().unwrap().y > e.points.last().unwrap().y, "{sub}'s inheritance edge runs upward to Base");
    }
    // Compartments sit inside their block, below the name, spanning its width.
    let engine = l.nodes["s-sys-engine"];
    let comp = l.nodes["s-sys-engine-compartment"];
    assert!(inside(&comp, &engine, 0.5), "{comp:?} in {engine:?}");
    assert_eq!(comp.w, engine.w);
    assert!(comp.y >= l.labels["s-sys-engine-label"].bottom(), "the compartment starts where the name label ends");
    assert!(l.labels.contains_key("s-sys-engine-compartment-line-0"));
    assert!(inside(&l.labels["s-sys-engine-compartment-line-1"], &comp, 0.5));
}

/// Every number as a float with one canonical spelling, so `1` and `1.0`
/// compare equal and the comparison tolerates nothing else.
fn canon(v: &Value) -> Value {
    match v {
        Value::Number(n) => Value::Number(Number::from_f64(n.as_f64().unwrap()).unwrap()),
        Value::Array(a) => Value::Array(a.iter().map(canon).collect()),
        Value::Object(o) => {
            let mut m = Map::new();
            for (k, x) in o {
                m.insert(k.clone(), canon(x));
            }
            Value::Object(m)
        }
        other => other.clone(),
    }
}

#[test]
fn elk_under_quickjs_matches_node_for_the_ibd_fixture() {
    // The committed output was produced from the committed input with the
    // vendored bundle under Node (from the repository root):
    //
    //   node -e 'const ELK=require("./crates/syscribe-server/frontend/node_modules/elkjs/lib/elk.bundled.js");
    //     const fs=require("fs"); const p="crates/syscribe-model/tests/vis_snapshots/elk/";
    //     new ELK().layout(JSON.parse(fs.readFileSync(p+"power_ibd.input.json","utf8")))
    //       .then(g=>fs.writeFileSync(p+"power_ibd.output.json", JSON.stringify(g,null,2)+"\n"))'
    let root = fixture_model();
    add_diagram(&root, "PowerIBD", "diagramKind: IBD\nsubject: Sys::PowerSystem\n");
    let g = graph_of(&root, "Diagrams::PowerIBD");
    let input = elk_input(&g, &sizes_of(&g));
    let input_path = snapshot_dir().join("power_ibd.input.json");
    let actual_input = serde_json::to_string_pretty(&input.graph).unwrap() + "\n";
    if std::env::var_os("SYSCRIBE_UPDATE_SNAPSHOTS").is_some() {
        std::fs::create_dir_all(snapshot_dir()).unwrap();
        std::fs::write(&input_path, &actual_input).unwrap();
    }
    let expected_input = std::fs::read_to_string(&input_path).unwrap_or_else(|e| panic!("missing {}: {e}", input_path.display()));
    assert_eq!(actual_input, expected_input, "the ELK input builder changed; refresh the snapshot and the Node output");

    let output_path = snapshot_dir().join("power_ibd.output.json");
    let expected: Value = serde_json::from_str(&std::fs::read_to_string(&output_path).unwrap_or_else(|e| panic!("missing {}: {e}", output_path.display()))).unwrap();
    let committed: Value = serde_json::from_str(&expected_input).unwrap();
    let actual = run_elk(&committed).expect("QuickJS layout");
    assert_eq!(
        serde_json::to_string_pretty(&canon(&actual)).unwrap(),
        serde_json::to_string_pretty(&canon(&expected)).unwrap(),
        "ELK under QuickJS and under Node disagree"
    );
    // The result carries real geometry, not an empty echo of the input.
    assert!(actual["children"][0]["children"].as_array().unwrap().iter().all(|c| c["x"].is_number() && c["width"].is_number()));
    assert!(actual["edges"].as_array().unwrap().iter().all(|e| e["sections"].is_array()));
}

fn synthetic_bdd(blocks: usize, edges: usize) -> DiagramGraph {
    let mut g = DiagramGraph::empty(DiagramKind::Bdd, "Diagrams::Big", "Big", Some("Sys"));
    for i in 0..blocks {
        g.nodes.push(Node {
            id: format!("n{i}"),
            element_ref: format!("Sys::Block{i}"),
            resolved: true,
            element_type: Some("PartDef".into()),
            kind: NodeKind::Block,
            label: format!("Block{i}"),
            stereotype: Some("part def".into()),
            parent: None,
            direction: None,
            side: None,
            lines: vec![],
            is_abstract: false,
            pin: None,
            banners: vec![],
            feature: None,
        });
        if i % 3 == 0 {
            g.nodes.push(Node {
                id: format!("n{i}-compartment"),
                element_ref: format!("Sys::Block{i}"),
                resolved: true,
                element_type: None,
                kind: NodeKind::Compartment,
                label: String::new(),
                stereotype: None,
                parent: Some(format!("n{i}")),
                direction: None,
                side: None,
                lines: vec![format!("attr{i} : Real"), "port p : PowerPort (out)".into()],
                is_abstract: false,
                pin: None,
                banners: vec![],
                feature: None,
            });
        }
    }
    // A deterministic mix: a tree of inheritance plus compositions and
    // associations across the tree, no self-loops.
    let mut state: u64 = 42;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for k in 0..edges {
        let (kind, label) = match k % 3 {
            0 => (EdgeKind::Inheritance, None),
            1 => (EdgeKind::Composition, Some(format!("part{k}"))),
            _ => (EdgeKind::Association, Some(format!("link{k}"))),
        };
        let s = (next() % blocks as u64) as usize;
        let t = if kind == EdgeKind::Inheritance { s / 2 } else { (next() % blocks as u64) as usize };
        if s == t {
            continue;
        }
        g.edges.push(Edge {
            id: format!("e{k}"),
            element_ref: None,
            source: format!("n{s}"),
            target: format!("n{t}"),
            kind,
            label,
            waypoints: None,
        });
    }
    g
}

#[test]
fn a_two_hundred_node_bdd_lays_out_within_the_bound() {
    let g = synthetic_bdd(200, 300);
    assert_eq!(g.nodes.iter().filter(|n| n.kind == NodeKind::Block).count(), 200);
    assert!(g.edges.len() >= 290, "{} edges after dropping self-loops", g.edges.len());
    let sizes = sizes_of(&g);
    // `SYSCRIBE_ELK_DUMP=<file>` writes the ELK input, to time Node on it.
    if let Some(path) = std::env::var_os("SYSCRIBE_ELK_DUMP") {
        std::fs::write(path, serde_json::to_string(&elk_input(&g, &sizes).graph).unwrap()).unwrap();
    }
    // Pay the per-thread engine load first so the timing is the layout's own.
    let warm = Instant::now();
    layout(&synthetic_bdd(2, 1), &sizes_of(&synthetic_bdd(2, 1))).expect("warm-up");
    let warm_up = warm.elapsed();
    let t = Instant::now();
    let l = layout(&g, &sizes).expect("layout");
    let elapsed = t.elapsed();
    println!("200 nodes / {} edges: layout {elapsed:?} (engine warm-up {warm_up:?})", g.edges.len());
    assert!(elapsed.as_secs_f64() < 10.0, "REQ-TRS-VIS-016 bounds a 200/300 layout at 2 s; this took {elapsed:?}");
    let (pairs, _) = check_layout(&g, &sizes, &l);
    assert_eq!(pairs, 200 * 199 / 2);
}

#[test]
fn pinned_nodes_keep_their_positions_and_a_fully_pinned_graph_uses_fixed() {
    let root = fixture_model();
    add_diagram(
        &root,
        "Pinned",
        "diagramKind: IBD\nsubject: Sys::PowerSystem\nlayout:\n  s-sys-powersystem: {x: 0, y: 0}\n  s-sys-powersystem-mainout: {x: 474, y: 100, w: 12, h: 12}\n  s-sys-powersystem-engine: {x: 40, y: 60, w: 160, h: 50}\n  s-sys-powersystem-engine-powerout: {x: 154, y: 19, w: 12, h: 12}\n  s-sys-powersystem-motor: {x: 300, y: 60, w: 160, h: 50}\n  s-sys-powersystem-motor-powerin: {x: -6, y: 19, w: 12, h: 12}\n  s-sys-powersystem-aux: {x: 300, y: 160, w: 160, h: 50}\n  s-sys-powersystem-aux-powerin: {x: -6, y: 19, w: 12, h: 12}\n",
    );
    let g = graph_of(&root, "Diagrams::Pinned");
    assert!(g.is_fully_pinned());
    let sizes = sizes_of(&g);
    let input = elk_input(&g, &sizes);
    assert!(input.all_pinned);
    assert_eq!(input.graph["layoutOptions"]["elk.algorithm"], "fixed");
    assert_eq!(input.graph["children"][0]["layoutOptions"]["elk.algorithm"], "fixed", "the compound boundary too");
    assert!(input.graph["layoutOptions"].get("elk.interactive").is_none());
    let l = layout(&g, &sizes).expect("fixed layout");
    assert_eq!(l.algorithm, "fixed");
    // Every pinned node keeps its parent-relative position and its pinned size.
    let boundary = l.nodes["s-sys-powersystem"];
    for (id, (x, y, w, h)) in [
        ("s-sys-powersystem-engine", (40.0, 60.0, 160.0, 50.0)),
        ("s-sys-powersystem-motor", (300.0, 60.0, 160.0, 50.0)),
        ("s-sys-powersystem-aux", (300.0, 160.0, 160.0, 50.0)),
    ] {
        let b = l.nodes[id];
        assert_eq!((b.x - boundary.x, b.y - boundary.y, b.w, b.h), (x, y, w, h), "{id}");
    }
    let engine = l.nodes["s-sys-powersystem-engine"];
    let pout = l.nodes["s-sys-powersystem-engine-powerout"];
    assert_eq!((pout.x - engine.x, pout.y - engine.y), (154.0, 19.0));
    // `fixed` routes straight lines and places no labels; the postprocessor
    // placed them as the client does (name under the top padding, centred).
    let name = l.labels["s-sys-powersystem-engine-label"];
    assert!((name.cx() - engine.cx()).abs() < 0.01 && name.y > engine.y);
    assert!(l.labels.contains_key("s-sys-powersystem-engine-powerout-label"));
    for e in l.edges.values() {
        assert!(!e.routed && e.points.len() == 2, "fixed: a centre-to-centre line");
    }

    // One pin among unpinned nodes: interactive layering around it.
    add_diagram(&root, "Partial", "diagramKind: IBD\nsubject: Sys::PowerSystem\nlayout:\n  s-sys-powersystem-aux: {x: 40, y: 200}\n");
    let g = graph_of(&root, "Diagrams::Partial");
    let sizes = sizes_of(&g);
    let input = elk_input(&g, &sizes);
    assert!(input.any_pinned && !input.all_pinned);
    assert_eq!(input.graph["layoutOptions"]["elk.algorithm"], "layered");
    assert_eq!(input.graph["layoutOptions"]["elk.interactive"], "true");
    assert_eq!(input.graph["layoutOptions"]["elk.layered.crossingMinimization.strategy"], "INTERACTIVE");
    let boundary = &input.graph["children"][0];
    assert_eq!(boundary["layoutOptions"]["elk.interactive"], "true", "compound nodes carry the interactive options too");
    let aux = boundary["children"].as_array().unwrap().iter().find(|c| c["id"] == "s-sys-powersystem-aux").unwrap();
    assert_eq!(aux["layoutOptions"]["elk.position"], "(40, 200)");
    assert_eq!(aux["x"], 40.0);
    let l = layout(&g, &sizes).expect("interactive layout");
    assert_eq!(l.algorithm, "layered");
    check_layout(&g, &sizes, &l);
    // The pinned node's placement drives the ordering: aux, pinned lowest,
    // stays below the unpinned engine and motor.
    assert!(l.nodes["s-sys-powersystem-aux"].y > l.nodes["s-sys-powersystem-engine"].y);
    assert!(l.nodes["s-sys-powersystem-aux"].y > l.nodes["s-sys-powersystem-motor"].y);
    let _ = Rect { x: 0.0, y: 0.0, w: None, h: None };
}
