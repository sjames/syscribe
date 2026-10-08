//! `REQ-TRS-VIS-021`: the Sequence generator through the real walker and
//! validator. A `Diagram` with `diagramKind: Sequence`, a `subject:` and no
//! `shapes:` derives its lifelines, messages, fragments and activation from
//! the subject `ActionDef`'s send/accept actions, places everything itself
//! (every node pinned, every message with horizontal waypoints) so each
//! renderer draws it with the `fixed` algorithm and no ELK run. The IR for
//! the fixture is pinned by the golden JSON `tests/vis_snapshots/derived/
//! mission_seq.json` — refresh with `SYSCRIBE_UPDATE_SNAPSHOTS=1 cargo test
//! -p syscribe-model --test vis_derive_sequence` and review the diff.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::plantuml::render_plantuml;
use syscribe_model::resolver::Resolver;
use syscribe_model::validator::{validate, Finding};
use syscribe_model::vis::derive::sequence::{ACTOR_H, HEADER_H, HEADER_W, LIFELINE_PITCH, ROW_PITCH};
use syscribe_model::vis::ir::LayoutAlgorithm;
use syscribe_model::vis::metrics::ApproxMetrics;
use syscribe_model::vis::svg::{pinned_layout, render_svg_with};
use syscribe_model::vis::{build_graph, render_mermaid, size_graph, DiagramGraph, EdgeKind, NodeKind};
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-vis-derive-seq-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// The fixture: a flight controller that performs `Mission` and owns the
/// `gpsIn` port, a propulsion part, a ground-station actor, and the
/// `Mission` action — two perform steps, a send `to` a part, an accept `via`
/// a port, an if with a send in each branch (the else to an unresolvable
/// chain), a loop with a body send, successions that reorder the
/// declaration, and `actors:`.
fn fixture_model() -> PathBuf {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "Sys/_index.md", "---\ntype: Package\nname: Sys\n---\n");
    write(&root, "Sys/DataPort.md", "---\ntype: PortDef\nname: DataPort\n---\n\nData port.\n");
    write(
        &root,
        "Sys/FlightController.md",
        "---\ntype: PartDef\nname: FlightController\nfeatures:\n  - name: gpsIn\n    type: Port\n    typedBy: Sys::DataPort\n    direction: in\nperforms:\n  - name: runMission\n    typedBy: Sys::Mission\n---\n\nFC.\n",
    );
    write(&root, "Sys/Propulsion.md", "---\ntype: PartDef\nname: Propulsion\n---\n\nMotors.\n");
    write(&root, "Sys/GroundStation.md", "---\ntype: PartDef\nname: GroundStation\n---\n\nGCS.\n");
    write(&root, "Sys/Takeoff.md", "---\ntype: ActionDef\nname: Takeoff\n---\n\nTakeoff.\n");
    write(&root, "Sys/Landing.md", "---\ntype: ActionDef\nname: Landing\n---\n\nLanding.\n");
    write(
        &root,
        "Sys/Mission.md",
        concat!(
            "---\ntype: ActionDef\nname: Mission\nactors:\n  - Sys::GroundStation\nsubActions:\n",
            "  - name: takeoff\n    kind: PerformAction\n    typedBy: Sys::Takeoff\n",
            "  - name: land\n    kind: PerformAction\n    typedBy: Sys::Landing\n",
            "  - name: setThrottle\n    kind: SendAction\n    payload: Sys::ThrottleCmd\n    to: Sys::Propulsion\n",
            "  - name: awaitFix\n    kind: AcceptAction\n    payload: Sys::GpsFix\n    via: gpsIn\n",
            "  - name: checkWind\n    kind: IfAction\n    condition: \"wind > 12\"\n    then:\n      - name: abort\n        kind: SendAction\n        payload: Sys::AbortCmd\n        to: Sys::Propulsion\n    else:\n      - name: notifyRelay\n        kind: SendAction\n        payload: Sys::Status\n        to: relay.link\n",
            "  - name: navigate\n    kind: LoopAction\n    loopKind: for\n    variable: wp\n    sequence: waypoints\n    body:\n      - name: steer\n        kind: SendAction\n        payload: Sys::SteerCmd\n        to: Sys::Propulsion\n",
            "successionConnections:\n",
            "  - after: takeoff\n    before: awaitFix\n",
            "  - after: awaitFix\n    before: setThrottle\n",
            "  - after: setThrottle\n    before: navigate\n",
            "  - after: navigate\n    before: checkWind\n",
            "  - after: checkWind\n    before: land\n",
            "---\n\nMission.\n",
        ),
    );
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

fn derived(root: &Path) -> (DiagramGraph, Vec<Finding>) {
    add_diagram(root, "MissionSeq", "diagramKind: Sequence\nsubject: Sys::Mission\nstatus: approved\n");
    graph_of(root, "Diagrams::MissionSeq")
}

fn lifelines(g: &DiagramGraph) -> Vec<(&str, NodeKind, &str, bool)> {
    g.nodes.iter().filter(|n| matches!(n.kind, NodeKind::Lifeline | NodeKind::Actor)).map(|n| (n.id.as_str(), n.kind, n.label.as_str(), n.resolved)).collect()
}

#[test]
fn derived_sequence_of_an_actiondef_matches_its_golden_ir() {
    let root = fixture_model();
    let (g, findings) = derived(&root);
    assert!(codes(&findings, "W417").is_empty() && codes(&findings, "W418").is_empty(), "{findings:?}");
    assert!(codes(&findings, "E405").is_empty());
    assert!(g.derived);
    assert_snapshot("mission_seq", &g);
}

#[test]
fn participants_appear_subject_first_in_first_appearance_order_then_actors() {
    let root = fixture_model();
    let (g, _) = derived(&root);
    assert_eq!(
        lifelines(&g),
        vec![
            ("s-sys-mission", NodeKind::Lifeline, "Mission", true),
            // `awaitFix via gpsIn` comes first in execution order: the port's owner, which performs Mission.
            ("s-sys-mission-sys-flightcontroller", NodeKind::Lifeline, "FlightController", true),
            ("s-sys-mission-sys-propulsion", NodeKind::Lifeline, "Propulsion", true),
            // `notifyRelay to relay.link` resolves to nothing: a dashed lifeline named by the chain.
            ("s-sys-mission-relay-link", NodeKind::Lifeline, "relay.link", false),
            ("s-sys-mission-sys-groundstation", NodeKind::Actor, "GroundStation", true),
        ]
    );
    assert_eq!(g.node("s-sys-mission").unwrap().stereotype.as_deref(), Some("action def"));
    assert_eq!(g.node("s-sys-mission-sys-propulsion").unwrap().stereotype.as_deref(), Some("part def"));
    assert_eq!(g.node("s-sys-mission-relay-link").unwrap().element_ref, "relay.link");
}

#[test]
fn messages_follow_the_successions_and_descend_into_branches_and_bodies() {
    let root = fixture_model();
    let (g, _) = derived(&root);
    // Declared: takeoff, land, setThrottle, awaitFix, checkWind, navigate.
    // Successions: takeoff → awaitFix → setThrottle → navigate → checkWind → land.
    let msgs: Vec<(&str, &str, &str, &str)> =
        g.edges.iter().map(|e| (e.id.as_str(), e.source.as_str(), e.target.as_str(), e.label.as_deref().unwrap_or(""))).collect();
    assert_eq!(
        msgs,
        vec![
            ("e-sys-mission-awaitfix", "s-sys-mission-sys-flightcontroller", "s-sys-mission", "awaitFix(GpsFix)"),
            ("e-sys-mission-setthrottle", "s-sys-mission", "s-sys-mission-sys-propulsion", "setThrottle(ThrottleCmd)"),
            ("e-sys-mission-steer", "s-sys-mission", "s-sys-mission-sys-propulsion", "steer(SteerCmd)"),
            ("e-sys-mission-abort", "s-sys-mission", "s-sys-mission-sys-propulsion", "abort(AbortCmd)"),
            ("e-sys-mission-notifyrelay", "s-sys-mission", "s-sys-mission-relay-link", "notifyRelay(Status)"),
        ]
    );
    assert!(g.edges.iter().all(|e| e.kind == EdgeKind::Message));
    assert_eq!(g.edges[0].element_ref.as_deref(), Some("Sys::Mission::awaitFix"));
    let rows: Vec<f64> = g.edges.iter().map(|e| e.waypoints.as_ref().unwrap()[0].y).collect();
    assert!(rows.windows(2).all(|w| w[1] > w[0]), "rows descend in execution order: {rows:?}");
}

#[test]
fn fragments_span_the_messages_they_contain() {
    let root = fixture_model();
    let (g, _) = derived(&root);
    let frags: Vec<(&str, &str, &str)> =
        g.nodes.iter().filter(|n| n.kind == NodeKind::Fragment).map(|n| (n.id.as_str(), n.stereotype.as_deref().unwrap_or(""), n.label.as_str())).collect();
    assert_eq!(frags, vec![("s-sys-mission-navigate", "loop", "[for wp in waypoints]"), ("s-sys-mission-checkwind", "alt", "[wind > 12]")]);
    let encloses = |frag: &str, edge: usize| {
        let p = g.node(frag).unwrap().pin.unwrap();
        let w = g.edges[edge].waypoints.as_ref().unwrap();
        let (lo, hi) = (w[0].x.min(w[1].x), w[0].x.max(w[1].x));
        p.x < lo && p.x + p.w.unwrap() > hi && p.y < w[0].y && p.y + p.h.unwrap() > w[0].y
    };
    assert!(encloses("s-sys-mission-navigate", 2), "loop around steer");
    assert!(encloses("s-sys-mission-checkwind", 3) && encloses("s-sys-mission-checkwind", 4), "alt around abort and notifyRelay");
    assert!(!encloses("s-sys-mission-navigate", 3), "the loop does not reach into the alt");
    assert!(g.nodes.iter().filter(|n| n.kind == NodeKind::Fragment).all(|n| n.parent.is_none()), "top-level fragments are roots");
}

#[test]
fn the_activation_sits_on_the_subjects_lifeline_spanning_its_messages() {
    let root = fixture_model();
    let (g, _) = derived(&root);
    let act = g.node("s-sys-mission-activation").expect("activation");
    assert_eq!(act.kind, NodeKind::Activation);
    assert_eq!(act.parent.as_deref(), Some("s-sys-mission"));
    let p = act.pin.unwrap();
    let first = g.edges.first().unwrap().waypoints.as_ref().unwrap()[0].y;
    let last = g.edges.last().unwrap().waypoints.as_ref().unwrap()[0].y;
    assert!(p.y < first && p.y + p.h.unwrap() > last);
    assert!(p.x > 0.0 && p.x + p.w.unwrap() < HEADER_W, "centred on the stem, parent-relative");
}

#[test]
fn every_node_is_pinned_and_every_message_runs_horizontally_between_stems() {
    let root = fixture_model();
    let (g, _) = derived(&root);
    assert!(g.is_fully_pinned());
    assert_eq!(g.layout_hints.algorithm, LayoutAlgorithm::Fixed);
    for (i, (id, kind, _, _)) in lifelines(&g).into_iter().enumerate() {
        let p = g.node(id).unwrap().pin.unwrap();
        assert_eq!((p.x, p.y, p.w, p.h), (i as f64 * LIFELINE_PITCH, 0.0, Some(HEADER_W), Some(if kind == NodeKind::Actor { ACTOR_H } else { HEADER_H })));
    }
    let stem = |id: &str| g.node(id).unwrap().pin.unwrap().x + HEADER_W / 2.0;
    for e in &g.edges {
        let w = e.waypoints.as_ref().expect("waypoints on every message");
        assert_eq!(w.len(), 2);
        assert_eq!(w[0].y, w[1].y, "horizontal");
        assert_eq!((w[0].x, w[1].x), (stem(&e.source), stem(&e.target)));
    }
    let rows: Vec<f64> = g.edges.iter().map(|e| e.waypoints.as_ref().unwrap()[0].y).collect();
    assert!(rows[0] > ACTOR_H, "the first row is below the tallest header");
    assert_eq!(rows[1] - rows[0], ROW_PITCH);
    // The pinned path draws it: no ELK run, every node and edge placed.
    let sizes = size_graph(&g, &ApproxMetrics);
    let laid = pinned_layout(&g, &sizes);
    assert_eq!(laid.algorithm, "pinned");
    assert_eq!(laid.nodes.len(), g.nodes.len());
    assert_eq!(laid.edges.len(), g.edges.len());
    let route = &laid.edges["e-sys-mission-setthrottle"];
    assert!(route.routed && route.points.len() == 2 && route.points[0].y == route.points[1].y, "stem to stem, not header to header: {route:?}");
}

#[test]
fn a_partdef_subject_is_w418_and_draws_nothing() {
    let root = fixture_model();
    add_diagram(&root, "Bad", "diagramKind: Sequence\nsubject: Sys::Propulsion\n");
    let (g, findings) = graph_of(&root, "Diagrams::Bad");
    assert!(g.nodes.is_empty());
    let w418 = codes(&findings, "W418");
    assert_eq!(w418.len(), 1, "{findings:?}");
    assert!(w418[0].contains("PartDef"));
}

#[test]
fn filters_keep_or_drop_participants_by_qualified_or_short_name() {
    let root = fixture_model();
    add_diagram(&root, "Some", "diagramKind: Sequence\nsubject: Sys::Mission\ninclude: [Sys::Propulsion, GroundStation, Ghost]\n");
    let (g, findings) = graph_of(&root, "Diagrams::Some");
    let ids: Vec<&str> = lifelines(&g).into_iter().map(|l| l.0).collect();
    assert_eq!(ids, vec!["s-sys-mission", "s-sys-mission-sys-propulsion", "s-sys-mission-sys-groundstation"], "the subject is never filtered");
    assert_eq!(g.edges.len(), 3, "only the messages to Propulsion remain");
    let w417 = codes(&findings, "W417");
    assert_eq!(w417.len(), 1, "{findings:?}");
    assert!(w417[0].contains("'Ghost'"));
    add_diagram(&root, "Less", "diagramKind: Sequence\nsubject: Sys::Mission\nexclude: [FlightController]\n");
    let (g, findings) = graph_of(&root, "Diagrams::Less");
    assert!(codes(&findings, "W417").is_empty());
    assert!(g.node("s-sys-mission-sys-flightcontroller").is_none());
    assert!(g.edges.iter().all(|e| e.id != "e-sys-mission-awaitfix"));
}

#[test]
fn a_derived_sequence_diagram_raises_no_w080_while_a_manifest_still_does() {
    let root = fixture_model();
    let (_g, findings) = derived(&root);
    assert!(codes(&findings, "W080").is_empty(), "the generator covers every message itself: {findings:?}");
    assert!(codes(&findings, "W402").is_empty() && codes(&findings, "W403").is_empty());
    add_diagram(
        &root,
        "Hand",
        "diagramKind: Sequence\nsubject: Sys::Mission\nstatus: approved\nshapes:\n  fc: {ref: Sys::FlightController, kind: lifeline}\n  prop: {ref: Sys::Propulsion, kind: lifeline}\nedges:\n  e1: {ref: Sys::Mission::setThrottle, source: fc, target: prop, kind: message}\n",
    );
    let (_g, findings) = graph_of(&root, "Diagrams::Hand");
    let w080 = codes(&findings, "W080");
    // The validator's walk covers `then`/`else` (§22.4), not a loop `body`, so
    // `steer` is not counted; the generator itself does descend into bodies.
    assert_eq!(w080.len(), 3, "awaitFix, abort, notifyRelay are unlisted: {w080:?}");
}

#[test]
fn every_writer_accepts_the_derived_sequence() {
    let root = fixture_model();
    let (g, _) = derived(&root);
    let no_links = |_: &str| None;
    let svg = render_svg_with(&g, &size_graph(&g, &ApproxMetrics), &no_links).expect("the pinned path draws it");
    assert!(svg.contains("class=\"syscribe-diagram Sequence\""), "{svg}");
    assert_eq!(svg.matches("stroke-dasharray=\"6,4\"").count(), 5, "one dashed stem per lifeline and actor");
    assert!(svg.contains("<path id=\"e-sys-mission-setthrottle\" class=\"edge message\""), "{svg}");
    assert!(svg.contains("<g id=\"s-sys-mission-activation\" class=\"activation\""));
    assert!(svg.contains("<g id=\"s-sys-mission-checkwind\" class=\"fragment\"") && svg.contains(">«alt»</text>") && svg.contains(">[wind &gt; 12]</text>"), "{svg}");
    assert!(svg.contains("<circle "), "the actor's stick figure");
    // The message runs along its row only: the path's two points share a y.
    let d = svg.split("id=\"e-sys-mission-setthrottle\"").nth(1).and_then(|s| s.split(" d=\"").nth(1)).and_then(|s| s.split('"').next()).unwrap();
    let ys: Vec<&str> = d.split(' ').filter(|t| t.contains(',')).map(|t| t.split(',').nth(1).unwrap()).collect();
    assert_eq!(ys.len(), 2);
    assert_eq!(ys[0], ys[1], "{d}");
    let mmd = render_mermaid(&g, &no_links).expect("mermaid");
    assert!(mmd.starts_with("sequenceDiagram\n"), "{mmd}");
    assert!(mmd.contains("actor ") && mmd.contains("participant ") && mmd.contains("->>"), "{mmd}");
    let elements = walk_model(&root).unwrap();
    let d = elements.iter().find(|e| e.qualified_name == "Diagrams::MissionSeq").unwrap();
    let puml = render_plantuml(d, &elements, None).expect("plantuml");
    assert!(puml.contains("@startuml") && puml.contains("participant ") && puml.contains("actor ") && puml.contains(" -> "), "{puml}");
}

// ── hand-listed sequence diagrams (REQ-TRS-VIS-025) ─────────────────────────

mod manifest_sequence {
    use std::path::Path;

    use syscribe_model::resolver::Resolver;
    use syscribe_model::vis::derive::sequence::{ACTIVATION_W, HEADER_H, HEADER_W, LIFELINE_PITCH, ROW_PITCH};
    use syscribe_model::vis::{build_graph, render_svg, DiagramGraph, EdgeKind, NodeKind};
    use syscribe_model::walker::walk_model;

    use super::{tempdir, write};

    const HAND_LISTED: &str = "---
type: Diagram
name: Hand
diagramKind: Sequence
shapes:
  ll-a: {ref: Sys::A, kind: actor}
  ll-b: {ref: Sys::B, kind: lifeline}
  ll-c: {ref: Sys::C, kind: lifeline}
  act-b1: {ref: Sys::B, kind: activation, parent: ll-b}
  act-b2: {ref: Sys::B, kind: activation, parent: ll-b}
  frag-outer: {ref: Sys::Flow, kind: fragment}
  frag-inner: {ref: Sys::Flow::inner, kind: fragment}
  note-1: {ref: Sys::A, kind: note}
edges:
  e1: {ref: Sys::Flow::start, source: ll-a, target: ll-b, kind: message}
  e2: {ref: Sys::Flow::inner::one, source: ll-b, target: ll-c, kind: message}
  e3: {ref: Sys::Flow::inner::two, source: ll-c, target: ll-b, kind: return, label: ack}
  e4: {ref: Sys::Flow::again, source: ll-b, target: ll-b, kind: message}
  e5: {ref: Sys::Flow::done, source: ll-b, target: act-b2, kind: message}
---

A hand-listed sequence diagram with no layout.
";

    fn model_with(diagram: &str) -> std::path::PathBuf {
        let root = tempdir();
        write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
        write(&root, "Sys/_index.md", "---\ntype: Package\nname: Sys\n---\n");
        for n in ["A", "B", "C"] {
            write(&root, &format!("Sys/{n}.md"), &format!("---\ntype: PartDef\nname: {n}\n---\n\n{n}.\n"));
        }
        write(&root, "Diagrams/Hand.md", diagram);
        root
    }

    fn graph(root: &Path) -> DiagramGraph {
        let elements = walk_model(root).unwrap();
        let resolver = Resolver::new(&elements);
        let d = elements.iter().find(|e| e.qualified_name == "Diagrams::Hand").unwrap();
        build_graph(d, &elements, &resolver).unwrap().0
    }

    fn pin(g: &DiagramGraph, id: &str) -> syscribe_model::vis::Rect {
        g.node(id).unwrap_or_else(|| panic!("node {id}")).pin.unwrap_or_else(|| panic!("{id} is pinned"))
    }

    #[test]
    fn a_hand_listed_sequence_diagram_is_fully_pinned_with_waypoints_on_every_message() {
        let g = graph(&model_with(HAND_LISTED));
        assert!(g.is_fully_pinned(), "every node is pinned: {:?}", g.nodes.iter().filter(|n| n.pin.is_none()).map(|n| &n.id).collect::<Vec<_>>());
        for e in &g.edges {
            assert!(e.waypoints.as_ref().is_some_and(|w| w.len() >= 2), "{} has waypoints", e.id);
        }
        // Columns: declaration order, fixed pitch, headers at the top.
        for (i, id) in ["ll-a", "ll-b", "ll-c"].iter().enumerate() {
            let p = pin(&g, id);
            assert_eq!((p.x, p.y, p.w), (i as f64 * LIFELINE_PITCH, 0.0, Some(HEADER_W)));
        }
        assert_eq!(pin(&g, "ll-b").h, Some(HEADER_H));
        // The stray note is pinned in a row below the diagram, not at the origin.
        let note = pin(&g, "note-1");
        assert!(note.y > pin(&g, "frag-outer").y + pin(&g, "frag-outer").h.unwrap(), "{note:?}");
    }

    #[test]
    fn messages_are_rows_in_declaration_order_stem_to_stem_and_self_messages_loop() {
        let g = graph(&model_with(HAND_LISTED));
        let stem = |c: usize| c as f64 * LIFELINE_PITCH + HEADER_W / 2.0;
        let wp = |id: &str| g.edges.iter().find(|e| e.id == id).unwrap().waypoints.clone().unwrap();
        let ys: Vec<f64> = ["e1", "e2", "e3", "e4", "e5"].iter().map(|id| wp(id)[0].y).collect();
        assert!(ys.windows(2).all(|w| w[1] - w[0] >= ROW_PITCH), "rows go down by at least a row pitch: {ys:?}");
        let e1 = wp("e1");
        assert_eq!((e1[0].x, e1[1].x, e1[0].y == e1[1].y), (stem(0), stem(1), true));
        let e3 = wp("e3");
        assert_eq!((e3[0].x, e3[1].x), (stem(2), stem(1)), "a return runs from its source stem to its target stem");
        assert_eq!(wp("e4").len(), 4, "a self message is the small loop");
        // An edge from a lifeline into its own activation resolves to the same
        // column, so it is a self message too: it leaves and returns to the stem.
        let e5 = wp("e5");
        assert_eq!((e5.len(), e5[0].x, e5[3].x), (4, stem(1), stem(1)));
    }

    #[test]
    fn unlabelled_messages_are_labelled_by_their_ref_and_labels_are_kept() {
        let g = graph(&model_with(HAND_LISTED));
        let label = |id: &str| g.edges.iter().find(|e| e.id == id).unwrap().label.clone();
        assert_eq!(label("e1").as_deref(), Some("start"));
        assert_eq!(label("e3").as_deref(), Some("ack"), "an author's label wins");
        assert!(g.edges.iter().all(|e| e.kind != EdgeKind::Message || e.label.is_some()));
    }

    #[test]
    fn fragments_box_their_messages_and_the_outer_one_is_wider() {
        let g = graph(&model_with(HAND_LISTED));
        let outer = pin(&g, "frag-outer");
        let inner = pin(&g, "frag-inner");
        let row = |id: &str| g.edges.iter().find(|e| e.id == id).unwrap().waypoints.as_ref().unwrap()[0].y;
        // `Sys::Flow` covers every message; `Sys::Flow::inner` covers e2 and e3 only.
        assert!(outer.y < row("e1") && row("e5") < outer.y + outer.h.unwrap(), "{outer:?}");
        assert!(inner.y < row("e2") && row("e3") < inner.y + inner.h.unwrap(), "{inner:?}");
        assert!(row("e1") < inner.y || row("e1") > inner.y + inner.h.unwrap(), "e1 is outside the inner fragment");
        assert!(inner.y > outer.y && inner.y + inner.h.unwrap() < outer.y + outer.h.unwrap(), "inner sits inside outer");
        assert!(outer.x < inner.x && outer.x + outer.w.unwrap() > inner.x + inner.w.unwrap(), "outer is wider than inner");
        let node = g.node("frag-outer").unwrap();
        assert_eq!(node.kind, NodeKind::Fragment);
    }

    #[test]
    fn several_activations_on_one_lifeline_share_its_rows_in_order() {
        let g = graph(&model_with(HAND_LISTED));
        let (a1, a2) = (pin(&g, "act-b1"), pin(&g, "act-b2"));
        assert_eq!((a1.x, a1.w), ((HEADER_W - ACTIVATION_W) / 2.0, Some(ACTIVATION_W)));
        assert!(a1.y + a1.h.unwrap() <= a2.y + 1.0, "the second activation starts after the first ends: {a1:?} {a2:?}");
        assert!(a2.y > a1.y);
    }

    #[test]
    fn a_diagram_with_any_pin_keeps_its_authors_geometry() {
        let with_pin = HAND_LISTED.replace("edges:", "layout:\n  ll-a: {x: 5, y: 6}\nedges:");
        let g = graph(&model_with(&with_pin));
        assert_eq!(pin(&g, "ll-a").x, 5.0);
        assert!(g.node("ll-b").unwrap().pin.is_none(), "nothing else was placed");
        assert!(g.edges.iter().all(|e| e.waypoints.is_none()));
    }

    #[test]
    fn it_renders_without_elk_and_never_fails_for_a_manifest_sequence_diagram() {
        let g = graph(&model_with(HAND_LISTED));
        let svg = render_svg(&g, &|_| None).expect("a placed sequence diagram draws");
        assert!(svg.contains("sysml:ref=\"Sys::Flow\""), "fragment drawn");
        assert!(svg.matches("class=\"edge ").count() >= 5, "messages drawn");
    }

    #[test]
    fn the_demo_models_hand_listed_sequence_diagram_exports() {
        let model = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../model");
        let elements = walk_model(&model).unwrap();
        let resolver = Resolver::new(&elements);
        let d = elements.iter().find(|e| e.qualified_name == "Diagrams::MissionExecutionSeq").expect("the demo diagram");
        let (g, issues) = build_graph(d, &elements, &resolver).unwrap();
        assert!(issues.is_empty(), "{issues:?}");
        assert!(g.is_fully_pinned());
        let svg = render_svg(&g, &|_| None).expect("the demo diagram exports (it failed with an ELK error before REQ-TRS-VIS-025)");
        assert!(svg.contains("FlightController"));
        // Lifeline headers carry no banners (they would overflow the fixed header).
        assert!(g.nodes.iter().filter(|n| n.kind == NodeKind::Lifeline).all(|n| n.banners.is_empty()));
    }
}
