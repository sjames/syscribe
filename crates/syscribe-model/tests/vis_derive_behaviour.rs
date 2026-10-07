//! `REQ-TRS-VIS-018`/`-019`: the derived StateMachine and Action diagrams
//! through the real walker and validator. A `StateDef` subject yields its
//! states, pseudostates and labelled transitions; an `ActionDef` subject its
//! steps, control nodes, branches, loop and successions. The generated IR for
//! the fixture model is pinned by golden JSON snapshots under
//! `tests/vis_snapshots/derived/` — refresh with
//! `SYSCRIBE_UPDATE_SNAPSHOTS=1 cargo test -p syscribe-model --test vis_derive_behaviour`
//! and review the diff. The writers are exercised on both graphs: Mermaid,
//! the static SVG (laid out by the embedded ELK) and, for the state machine,
//! PlantUML.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::plantuml::render_plantuml;
use syscribe_model::resolver::Resolver;
use syscribe_model::validator::{validate, Finding};
use syscribe_model::vis::{build_graph, render_mermaid, render_svg, DiagramGraph, EdgeKind, NodeKind};
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-vis-behaviour-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// The fixture: a `Flight` state machine (nested transitions in both accept
/// spellings, one top-level transition in the deprecated `from`/`to`/
/// `trigger` aliases, entry and do actions, an initial and a final state, and
/// a substate typed by a machine of its own), a `Mission` action (perform,
/// send and accept steps, an if/else, a loop with a body, fork and join
/// control nodes, successions and a flow), the action definitions they
/// reference, and a `PartDef` for the wrong-subject case.
fn fixture_model() -> PathBuf {
    let root = tempdir();
    write(&root, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&root, "Beh/_index.md", "---\ntype: Package\nname: Beh\n---\n");
    write(&root, "Beh/Takeoff.md", "---\ntype: ActionDef\nname: Takeoff\n---\n\nTakeoff.\n");
    write(&root, "Beh/Land.md", "---\ntype: ActionDef\nname: Land\n---\n\nLand.\n");
    write(&root, "Beh/Nav.md", "---\ntype: ActionDef\nname: Nav\n---\n\nNavigate.\n");
    write(&root, "Beh/Command.md", "---\ntype: ItemDef\nname: Command\n---\n\nCommand.\n");
    write(&root, "Beh/Fix.md", "---\ntype: ItemDef\nname: Fix\n---\n\nFix.\n");
    write(&root, "Beh/Airframe.md", "---\ntype: PartDef\nname: Airframe\n---\n\nNot a behaviour.\n");
    write(
        &root,
        "Beh/Cruise.md",
        "---\ntype: StateDef\nname: Cruise\nsubStates:\n  - name: hold\n    isInitial: true\n    transitions:\n      - target: track\n        accept: Beh::Fix\n  - name: track\n    isFinal: true\n---\n\nCruise sub-machine.\n",
    );
    write(
        &root,
        "Beh/Flight.md",
        "---\ntype: StateDef\nname: Flight\nsubStates:\n  - name: disarmed\n    isInitial: true\n    transitions:\n      - target: armed\n        accept:\n          payload: Beh::Command\n        guard: \"armed == false\"\n  - name: armed\n    entryAction: Beh::Takeoff\n    transitions:\n      - target: flying\n        accept: Beh::Command\n        guard: \"ready\"\n        effect:\n          name: startTakeoff\n          typedBy: Beh::Takeoff\n  - name: flying\n    typedBy: Beh::Cruise\n    doAction:\n      name: navigate\n      typedBy: Beh::Nav\n    transitions:\n      - target: landed\n        guard: \"altitude <= 0.1\"\n        effect: Beh::Land\n      - target: ghost\n        guard: \"never\"\n  - name: landed\n    isFinal: true\n    exitAction: Beh::Land\ntransitions:\n  - name: abort\n    from: flying\n    to: disarmed\n    trigger: Beh::Command\n---\n\nFlight states.\n",
    );
    write(
        &root,
        "Beh/Mission.md",
        "---\ntype: ActionDef\nname: Mission\nsubActions:\n  - name: takeoff\n    kind: PerformAction\n    typedBy: Beh::Takeoff\n  - name: checkWeather\n    kind: IfAction\n    condition: \"wind > 12\"\n    then:\n      - name: abort\n        kind: SendAction\n        payload: Beh::Command\n        via: ctrlOut\n    else:\n      - name: proceed\n        kind: PerformAction\n        typedBy: Beh::Nav\n  - name: navigate\n    kind: LoopAction\n    loopKind: for\n    variable: wp\n    sequence: waypoints\n    body:\n      - name: awaitArrival\n        kind: AcceptAction\n        payload: Beh::Fix\n        trigger:\n          kind: change\n          condition: \"near(wp)\"\n      - name: advance\n        kind: Action\n  - name: land\n    kind: PerformAction\n    typedBy: Beh::Land\ncontrolNodes:\n  - name: start\n    kind: ForkNode\n  - name: end\n    kind: JoinNode\nsuccessionConnections:\n  - after: start\n    before: takeoff\n  - after: takeoff\n    before: checkWeather\n  - after: checkWeather\n    before: navigate\n    guard: \"ok\"\n  - after: navigate\n    before: land\n  - after: land\n    before: end\nflowConnections:\n  - from: takeoff.alt\n    to: navigate.alt\n---\n\nMission.\n",
    );
    write(
        &root,
        "Beh/Unordered.md",
        "---\ntype: ActionDef\nname: Unordered\nsubActions:\n  - name: a\n    kind: Action\n  - name: b\n    kind: Action\n---\n\nNo successions.\n",
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

type EdgeTuple = (EdgeKind, String, String, Option<String>);

fn edges_of(g: &DiagramGraph) -> Vec<EdgeTuple> {
    g.edges.iter().map(|e| (e.kind, e.source.clone(), e.target.clone(), e.label.clone())).collect()
}

fn t(s: &str, tgt: &str, label: Option<&str>) -> EdgeTuple {
    (EdgeKind::Transition, s.to_string(), tgt.to_string(), label.map(str::to_string))
}

fn succ(s: &str, tgt: &str, label: Option<&str>) -> EdgeTuple {
    (EdgeKind::Succession, s.to_string(), tgt.to_string(), label.map(str::to_string))
}

fn no_links(_: &str) -> Option<String> {
    None
}

// ── StateMachine (REQ-TRS-VIS-018) ─────────────────────────────────────────

#[test]
fn derived_state_machine_of_a_statedef_matches_its_golden_ir() {
    let root = fixture_model();
    add_diagram(&root, "FlightSM", "diagramKind: StateMachine\nsubject: Beh::Flight\n");
    let (g, findings) = graph_of(&root, "Diagrams::FlightSM");
    assert!(codes(&findings, "W417").is_empty() && codes(&findings, "W418").is_empty(), "{findings:?}");
    assert!(codes(&findings, "W402").is_empty() && codes(&findings, "W403").is_empty(), "{findings:?}");
    assert!(g.derived);
    let states: Vec<&str> = g.nodes.iter().filter(|n| n.kind == NodeKind::State && n.parent.is_none()).map(|n| n.id.as_str()).collect();
    assert_eq!(states, vec!["s-beh-flight-disarmed", "s-beh-flight-armed", "s-beh-flight-flying", "s-beh-flight-landed"]);
    assert_eq!(g.node("s-beh-flight-armed").unwrap().label, "armed");
    assert_snapshot("flight_sm", &g);
}

#[test]
fn state_compartments_carry_entry_do_and_exit_actions() {
    let root = fixture_model();
    add_diagram(&root, "FlightSM", "diagramKind: StateMachine\nsubject: Beh::Flight\n");
    let (g, _) = graph_of(&root, "Diagrams::FlightSM");
    let comp = g.node("s-beh-flight-armed-compartment").unwrap();
    assert_eq!(comp.kind, NodeKind::Compartment);
    assert_eq!(comp.parent.as_deref(), Some("s-beh-flight-armed"));
    assert_eq!(comp.lines, vec!["entry / Takeoff"], "string form: the last `::` segment");
    assert_eq!(g.node("s-beh-flight-flying-compartment").unwrap().lines, vec!["do / navigate"], "map form: its name");
    assert_eq!(g.node("s-beh-flight-landed-compartment").unwrap().lines, vec!["exit / Land"]);
    assert!(g.node("s-beh-flight-disarmed-compartment").is_none(), "no actions, no compartment");
}

#[test]
fn transition_labels_are_accept_guard_effect_in_either_placement_and_spelling() {
    let root = fixture_model();
    add_diagram(&root, "FlightSM", "diagramKind: StateMachine\nsubject: Beh::Flight\n");
    let (g, _) = graph_of(&root, "Diagrams::FlightSM");
    let edges = edges_of(&g);
    // Nested, accept map + guard.
    assert!(edges.contains(&t("s-beh-flight-disarmed", "s-beh-flight-armed", Some("Command [armed == false]"))), "{edges:?}");
    // Nested, accept string + guard + effect map (its name).
    assert!(edges.contains(&t("s-beh-flight-armed", "s-beh-flight-flying", Some("Command [ready] / startTakeoff"))));
    // Nested, guard + effect string (last segment).
    assert!(edges.contains(&t("s-beh-flight-flying", "s-beh-flight-landed", Some("[altitude <= 0.1] / Land"))));
    // Top-level with the deprecated from/to/trigger aliases: accept only.
    assert!(edges.contains(&t("s-beh-flight-flying", "s-beh-flight-disarmed", Some("Command"))));
    let abort = g.edges.iter().find(|e| e.source == "s-beh-flight-flying" && e.target == "s-beh-flight-disarmed").unwrap();
    assert_eq!(abort.element_ref.as_deref(), Some("Beh::Flight::abort"), "a named transition is referenced by its name");
    assert_eq!(abort.id, "e-transition-s-beh-flight-flying-s-beh-flight-disarmed");
    assert!(g.edges.iter().all(|e| e.kind == EdgeKind::Transition));
}

#[test]
fn initial_feeds_the_initial_state_and_the_final_state_reaches_final() {
    let root = fixture_model();
    add_diagram(&root, "FlightSM", "diagramKind: StateMachine\nsubject: Beh::Flight\n");
    let (g, _) = graph_of(&root, "Diagrams::FlightSM");
    let initial = g.node("s-beh-flight-initial").unwrap();
    assert_eq!(initial.kind, NodeKind::Initial);
    assert!(initial.parent.is_none());
    assert_eq!(g.node("s-beh-flight-final").unwrap().kind, NodeKind::Final);
    let edges = edges_of(&g);
    assert!(edges.contains(&t("s-beh-flight-initial", "s-beh-flight-disarmed", None)), "{edges:?}");
    assert!(edges.contains(&t("s-beh-flight-landed", "s-beh-flight-final", None)));
    assert_eq!(g.nodes.iter().filter(|n| n.kind == NodeKind::Initial && n.parent.is_none()).count(), 1);
}

#[test]
fn a_substate_typed_by_a_machine_is_a_container_with_its_own_region() {
    let root = fixture_model();
    add_diagram(&root, "FlightSM", "diagramKind: StateMachine\nsubject: Beh::Flight\n");
    let (g, _) = graph_of(&root, "Diagrams::FlightSM");
    let inner: Vec<&str> = g.children_of("s-beh-flight-flying").filter(|n| n.kind == NodeKind::State).map(|n| n.id.as_str()).collect();
    assert_eq!(inner, vec!["s-beh-flight-flying-hold", "s-beh-flight-flying-track"]);
    assert_eq!(g.node("s-beh-flight-flying-initial").unwrap().parent.as_deref(), Some("s-beh-flight-flying"));
    assert_eq!(g.node("s-beh-flight-flying-final").unwrap().parent.as_deref(), Some("s-beh-flight-flying"));
    let edges = edges_of(&g);
    assert!(edges.contains(&t("s-beh-flight-flying-initial", "s-beh-flight-flying-hold", None)));
    assert!(edges.contains(&t("s-beh-flight-flying-hold", "s-beh-flight-flying-track", Some("Fix"))));
    assert!(edges.contains(&t("s-beh-flight-flying-track", "s-beh-flight-flying-final", None)));
}

#[test]
fn a_transition_to_a_missing_state_draws_no_edge_and_is_left_to_w929() {
    let root = fixture_model();
    add_diagram(&root, "FlightSM", "diagramKind: StateMachine\nsubject: Beh::Flight\n");
    let (g, findings) = graph_of(&root, "Diagrams::FlightSM");
    assert!(g.edges.iter().all(|e| !e.target.contains("ghost")));
    assert!(codes(&findings, "W418").is_empty() && codes(&findings, "W417").is_empty(), "the diagram is clean: {findings:?}");
    // 4 top-level transitions + initial/final + 3 in the nested region.
    assert_eq!(g.edges.len(), 9, "{:?}", edges_of(&g));
}

#[test]
fn state_filters_apply_by_name_or_qualified_name_and_a_stray_entry_is_w417() {
    let root = fixture_model();
    add_diagram(&root, "Some", "diagramKind: StateMachine\nsubject: Beh::Flight\ninclude: [disarmed, Beh::Flight::armed, limbo]\n");
    let (g, findings) = graph_of(&root, "Diagrams::Some");
    let states: Vec<&str> = g.nodes.iter().filter(|n| n.kind == NodeKind::State).map(|n| n.id.as_str()).collect();
    assert_eq!(states, vec!["s-beh-flight-disarmed", "s-beh-flight-armed"]);
    assert!(g.node("s-beh-flight-final").is_none());
    assert_eq!(g.edges.len(), 2, "initial→disarmed and disarmed→armed: {:?}", edges_of(&g));
    let w417 = codes(&findings, "W417");
    assert_eq!(w417.len(), 1, "{findings:?}");
    assert!(w417[0].contains("'limbo'"));

    add_diagram(&root, "Less", "diagramKind: StateMachine\nsubject: Beh::Flight\nexclude: [flying]\n");
    let (g, findings) = graph_of(&root, "Diagrams::Less");
    assert!(codes(&findings, "W417").is_empty());
    assert!(g.node("s-beh-flight-flying").is_none() && g.node("s-beh-flight-flying-hold").is_none());
    assert!(g.edges.iter().all(|e| !e.source.contains("flying") && !e.target.contains("flying")));
}

#[test]
fn a_partdef_subject_for_a_state_machine_is_w418_and_draws_nothing() {
    let root = fixture_model();
    add_diagram(&root, "Bad", "diagramKind: StateMachine\nsubject: Beh::Airframe\n");
    let (g, findings) = graph_of(&root, "Diagrams::Bad");
    assert!(g.nodes.is_empty());
    let w418 = codes(&findings, "W418");
    assert_eq!(w418.len(), 1, "{findings:?}");
    assert!(w418[0].contains("PartDef"));
}

#[test]
fn the_writers_accept_a_derived_state_machine() {
    let root = fixture_model();
    add_diagram(&root, "FlightSM", "diagramKind: StateMachine\nsubject: Beh::Flight\n");
    let (g, _) = graph_of(&root, "Diagrams::FlightSM");
    let mermaid = render_mermaid(&g, &no_links).expect("mermaid");
    assert!(mermaid.starts_with("stateDiagram-v2\n"), "{mermaid}");
    assert!(mermaid.contains("state \"flying\" as s_beh_flight_flying {\n"), "a composite state nests its region: {mermaid}");
    assert!(mermaid.contains("s_beh_flight_armed : entry / Takeoff"), "{mermaid}");
    assert!(mermaid.contains("[*] --> s_beh_flight_disarmed\n"), "{mermaid}");
    assert!(mermaid.contains("s_beh_flight_landed --> [*]\n"), "{mermaid}");
    assert!(mermaid.contains("s_beh_flight_armed --> s_beh_flight_flying : Command [ready] / startTakeoff\n"), "{mermaid}");
    let svg = render_svg(&g, &no_links).expect("svg laid out by the embedded ELK");
    assert!(svg.contains("class=\"state\"") || svg.contains("class=\"state State\""), "{}", &svg[..svg.len().min(600)]);
    assert!(svg.contains("sysml:ref=\"Beh::Flight::armed\""));
    assert!(svg.contains("class=\"edge transition\""));
    assert!(svg.contains("Command [ready] / startTakeoff"));
    let elements = walk_model(&root).unwrap();
    let d = elements.iter().find(|e| e.qualified_name == "Diagrams::FlightSM").unwrap();
    let puml = render_plantuml(d, &elements, None).expect("plantuml");
    assert!(puml.contains("state \"armed\" as s_beh_flight_armed"), "{puml}");
    assert!(puml.contains("s_beh_flight_armed : entry / Takeoff"), "{puml}");
    assert!(puml.contains("[*] --> s_beh_flight_disarmed\n"), "{puml}");
    assert!(puml.contains("s_beh_flight_landed --> [*]\n"), "{puml}");
    assert!(puml.contains("state \"flying\" as s_beh_flight_flying {\n"), "{puml}");
}

// ── Action (REQ-TRS-VIS-019) ───────────────────────────────────────────────

#[test]
fn derived_action_of_an_actiondef_matches_its_golden_ir() {
    let root = fixture_model();
    add_diagram(&root, "MissionAction", "diagramKind: Action\nsubject: Beh::Mission\n");
    let (g, findings) = graph_of(&root, "Diagrams::MissionAction");
    assert!(codes(&findings, "W417").is_empty() && codes(&findings, "W418").is_empty(), "{findings:?}");
    assert!(codes(&findings, "W400").is_empty(), "the Action kind is known to the validator: {findings:?}");
    assert!(codes(&findings, "W402").is_empty() && codes(&findings, "W403").is_empty(), "{findings:?}");
    assert!(g.derived);
    let steps: Vec<(&str, Option<&str>)> = g
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Action && n.parent.is_none())
        .map(|n| (n.id.as_str(), n.stereotype.as_deref()))
        .collect();
    // The if's branch steps are siblings of the decision (only a loop nests its body).
    assert_eq!(
        steps,
        vec![
            ("s-beh-mission-takeoff", Some("perform")),
            ("s-beh-mission-checkweather-abort", Some("send")),
            ("s-beh-mission-checkweather-proceed", Some("perform")),
            ("s-beh-mission-navigate", Some("loop")),
            ("s-beh-mission-land", Some("perform")),
        ]
    );
    assert_snapshot("mission_action", &g);
}

#[test]
fn action_steps_carry_their_kind_stereotype_and_compartment_lines() {
    let root = fixture_model();
    add_diagram(&root, "MissionAction", "diagramKind: Action\nsubject: Beh::Mission\n");
    let (g, _) = graph_of(&root, "Diagrams::MissionAction");
    assert_eq!(g.node("s-beh-mission-takeoff-compartment").unwrap().lines, vec![": Takeoff"]);
    let abort = g.node("s-beh-mission-checkweather-abort").unwrap();
    assert_eq!((abort.kind, abort.stereotype.as_deref(), abort.label.as_str()), (NodeKind::Action, Some("send"), "abort"));
    assert_eq!(g.node("s-beh-mission-checkweather-abort-compartment").unwrap().lines, vec!["send Command", "via ctrlOut"]);
    let await_ = g.node("s-beh-mission-navigate-awaitarrival").unwrap();
    assert_eq!(await_.stereotype.as_deref(), Some("accept"));
    assert_eq!(g.node("s-beh-mission-navigate-awaitarrival-compartment").unwrap().lines, vec!["accept Fix", "when near(wp)"]);
    assert!(g.node("s-beh-mission-navigate-advance-compartment").is_none(), "a bare action has no compartment");
}

#[test]
fn an_if_action_is_a_decision_with_then_else_branches_closed_by_a_merge() {
    let root = fixture_model();
    add_diagram(&root, "MissionAction", "diagramKind: Action\nsubject: Beh::Mission\n");
    let (g, _) = graph_of(&root, "Diagrams::MissionAction");
    let decision = g.node("s-beh-mission-checkweather").unwrap();
    assert_eq!((decision.kind, decision.label.as_str()), (NodeKind::Decision, "wind > 12"));
    assert_eq!(g.node("s-beh-mission-checkweather-merge").unwrap().kind, NodeKind::Merge);
    let edges = edges_of(&g);
    assert!(edges.contains(&succ("s-beh-mission-checkweather", "s-beh-mission-checkweather-abort", Some("[then]"))), "{edges:?}");
    assert!(edges.contains(&succ("s-beh-mission-checkweather", "s-beh-mission-checkweather-proceed", Some("[else]"))));
    assert!(edges.contains(&succ("s-beh-mission-checkweather-abort", "s-beh-mission-checkweather-merge", None)));
    assert!(edges.contains(&succ("s-beh-mission-checkweather-proceed", "s-beh-mission-checkweather-merge", None)));
    // Declared successions enter the if at its decision and leave from its merge, with the guard as label.
    assert!(edges.contains(&succ("s-beh-mission-takeoff", "s-beh-mission-checkweather", None)));
    assert!(edges.contains(&succ("s-beh-mission-checkweather-merge", "s-beh-mission-navigate", Some("[ok]"))));
}

#[test]
fn a_loop_action_is_a_container_holding_its_body_in_order() {
    let root = fixture_model();
    add_diagram(&root, "MissionAction", "diagramKind: Action\nsubject: Beh::Mission\n");
    let (g, _) = graph_of(&root, "Diagrams::MissionAction");
    let nav = g.node("s-beh-mission-navigate").unwrap();
    assert_eq!((nav.kind, nav.stereotype.as_deref(), nav.label.as_str()), (NodeKind::Action, Some("loop"), "navigate [for wp in waypoints]"));
    let body: Vec<&str> = g.children_of("s-beh-mission-navigate").filter(|n| n.kind == NodeKind::Action).map(|n| n.id.as_str()).collect();
    assert_eq!(body, vec!["s-beh-mission-navigate-awaitarrival", "s-beh-mission-navigate-advance"]);
    assert!(edges_of(&g).contains(&succ("s-beh-mission-navigate-awaitarrival", "s-beh-mission-navigate-advance", None)));
}

#[test]
fn control_nodes_successions_and_flows_come_from_the_subjects_lists() {
    let root = fixture_model();
    add_diagram(&root, "MissionAction", "diagramKind: Action\nsubject: Beh::Mission\n");
    let (g, _) = graph_of(&root, "Diagrams::MissionAction");
    assert_eq!(g.node("s-beh-mission-start").unwrap().kind, NodeKind::Fork);
    assert_eq!(g.node("s-beh-mission-end").unwrap().kind, NodeKind::Join);
    let edges = edges_of(&g);
    assert!(edges.contains(&succ("s-beh-mission-start", "s-beh-mission-takeoff", None)), "{edges:?}");
    assert!(edges.contains(&succ("s-beh-mission-navigate", "s-beh-mission-land", None)));
    assert!(edges.contains(&succ("s-beh-mission-land", "s-beh-mission-end", None)));
    assert!(edges.contains(&(EdgeKind::Flow, "s-beh-mission-takeoff".into(), "s-beh-mission-navigate".into(), None)), "a pin chain names its step");
    assert!(g.edges.iter().any(|e| e.id == "e-succession-s-beh-mission-checkweather-merge-s-beh-mission-navigate"));
}

#[test]
fn initial_and_final_bracket_the_flow_only_when_successions_exist() {
    let root = fixture_model();
    add_diagram(&root, "MissionAction", "diagramKind: Action\nsubject: Beh::Mission\n");
    let (g, _) = graph_of(&root, "Diagrams::MissionAction");
    assert_eq!(g.nodes[0].kind, NodeKind::Initial);
    assert_eq!(g.nodes[0].id, "s-beh-mission-initial");
    assert_eq!(g.node("s-beh-mission-final").unwrap().kind, NodeKind::Final);
    let edges = edges_of(&g);
    assert!(edges.contains(&succ("s-beh-mission-initial", "s-beh-mission-start", None)), "the fork has no incoming succession: {edges:?}");
    assert!(edges.contains(&succ("s-beh-mission-end", "s-beh-mission-final", None)), "the join has no outgoing one");
    assert_eq!(g.edges.iter().filter(|e| e.source == "s-beh-mission-initial").count(), 1);
    assert_eq!(g.edges.iter().filter(|e| e.target == "s-beh-mission-final").count(), 1);

    add_diagram(&root, "Steps", "diagramKind: Action\nsubject: Beh::Unordered\n");
    let (g, findings) = graph_of(&root, "Diagrams::Steps");
    assert!(codes(&findings, "W418").is_empty());
    assert_eq!(g.nodes.iter().filter(|n| n.kind == NodeKind::Action).count(), 2);
    assert!(g.nodes.iter().all(|n| !matches!(n.kind, NodeKind::Initial | NodeKind::Final)), "no successions, no initial/final");
    assert!(g.edges.is_empty());
}

#[test]
fn action_filters_apply_to_steps_and_control_nodes_and_a_stray_entry_is_w417() {
    let root = fixture_model();
    add_diagram(&root, "Some", "diagramKind: Action\nsubject: Beh::Mission\ninclude: [takeoff, Beh::Mission::land, ghost]\n");
    let (g, findings) = graph_of(&root, "Diagrams::Some");
    let steps: Vec<&str> = g.nodes.iter().filter(|n| n.kind == NodeKind::Action).map(|n| n.id.as_str()).collect();
    assert_eq!(steps, vec!["s-beh-mission-takeoff", "s-beh-mission-land"]);
    assert!(g.node("s-beh-mission-start").is_none(), "control nodes are filtered too");
    let w417 = codes(&findings, "W417");
    assert_eq!(w417.len(), 1, "{findings:?}");
    assert!(w417[0].contains("'ghost'"));

    add_diagram(&root, "Less", "diagramKind: Action\nsubject: Beh::Mission\nexclude: [checkWeather]\n");
    let (g, findings) = graph_of(&root, "Diagrams::Less");
    assert!(codes(&findings, "W417").is_empty());
    assert!(g.node("s-beh-mission-checkweather").is_none() && g.node("s-beh-mission-checkweather-abort").is_none());
    assert!(g.edges.iter().all(|e| !e.source.contains("checkweather") && !e.target.contains("checkweather")));
}

#[test]
fn a_partdef_subject_for_an_action_diagram_is_w418_and_draws_nothing() {
    let root = fixture_model();
    add_diagram(&root, "Bad", "diagramKind: Action\nsubject: Beh::Airframe\n");
    let (g, findings) = graph_of(&root, "Diagrams::Bad");
    assert!(g.nodes.is_empty());
    let w418 = codes(&findings, "W418");
    assert_eq!(w418.len(), 1, "{findings:?}");
    assert!(w418[0].contains("PartDef"));
}

#[test]
fn the_writers_accept_a_derived_action_diagram() {
    let root = fixture_model();
    add_diagram(&root, "MissionAction", "diagramKind: Action\nsubject: Beh::Mission\n");
    let (g, _) = graph_of(&root, "Diagrams::MissionAction");
    let mermaid = render_mermaid(&g, &no_links).expect("mermaid");
    assert!(mermaid.starts_with("flowchart TD\n"), "{mermaid}");
    assert!(mermaid.contains("s_beh_mission_checkweather{{\"wind > 12\"}}"), "a decision is a diamond: {mermaid}");
    assert!(mermaid.contains("s_beh_mission_start[[\"start\"]]"), "a fork is a bar: {mermaid}");
    assert!(mermaid.contains("subgraph s_beh_mission_navigate[\"navigate [for wp in waypoints]\"]"), "a loop is a subgraph: {mermaid}");
    assert!(mermaid.contains("s_beh_mission_checkweather -->|[then]| s_beh_mission_checkweather_abort"), "{mermaid}");
    let svg = render_svg(&g, &no_links).expect("svg laid out by the embedded ELK");
    assert!(svg.contains("class=\"decision\""), "{}", &svg[..svg.len().min(600)]);
    assert!(svg.contains("class=\"fork\"") && svg.contains("class=\"join\"") && svg.contains("class=\"merge\""));
    assert!(svg.contains("class=\"initial\"") && svg.contains("class=\"final\""));
    assert!(svg.contains("class=\"edge succession\""));
    assert!(svg.contains("sysml:ref=\"Beh::Mission::checkWeather::abort\""));
    assert!(svg.contains("[then]") && svg.contains("[ok]"));
    let elements = walk_model(&root).unwrap();
    let d = elements.iter().find(|e| e.qualified_name == "Diagrams::MissionAction").unwrap();
    assert!(render_plantuml(d, &elements, None).is_none(), "the PlantUML writer declines the Action kind");
}
