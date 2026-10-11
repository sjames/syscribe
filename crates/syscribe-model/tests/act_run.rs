//! TC-TRS-ACTEXEC-001 / REQ-TRS-ACTEXEC-001 (GH #276): the action-flow token interpreter.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::act_ir::{lower_action, ActionProgram};
use syscribe_model::act_run::{ActOptions, ActRun, Status};
use syscribe_model::expr::{format_value, parse_value, Value};
use syscribe_model::walker::walk_model;

fn program(files: &[(&str, &str)], root: &str) -> ActionProgram {
    static N: AtomicU64 = AtomicU64::new(0);
    let dir: PathBuf = std::env::temp_dir().join(format!("syscribe-actrun-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    for (n, body) in files {
        std::fs::write(dir.join(format!("{n}.md")), format!("---\ntype: ActionDef\nname: {n}\n{body}---\n")).unwrap();
    }
    lower_action(&walk_model(&dir).unwrap(), root).unwrap_or_else(|d| panic!("lowering failed: {d:?}"))
}

fn val(s: &str) -> Value {
    parse_value(s).unwrap()
}

fn num(r: &ActRun, name: &str) -> String {
    format_value(&r.var(name).unwrap_or_else(|| panic!("{name} unset")))
}

const B: &str = "parameters:
  - {name: x, direction: in}
  - {name: y, direction: out}
subActions:
  - {name: calc, kind: AssignmentAction, assign: \"y := x * 3\"}
";

const A: &str = "parameters:
  - {name: n, direction: in}
  - {name: result, direction: out}
variables:
  - {name: acc, value: \"0\"}
subActions:
  - {name: init, kind: AssignmentAction, assign: \"acc := n\"}
  - name: call
    kind: PerformAction
    typedBy: B
    parameters:
      - {name: x, value: acc}
  - {name: big, kind: AssignmentAction, assign: \"result := call.y\"}
  - {name: small, kind: AssignmentAction, assign: \"result := acc\"}
controlNodes:
  - {name: d, kind: DecisionNode}
successionConnections:
  - {after: init, before: call}
  - {after: call, before: d}
  - {after: d, before: big, guard: \"call.y > 10\"}
  - {after: d, before: small, guard: else}
";

fn run_a(n: &str) -> (ActionProgram, Vec<String>, String) {
    let p = program(&[("A", A), ("B", B)], "A");
    let mut r = ActRun::new(&p, ActOptions::default());
    r.set_input("n", val(n)).unwrap();
    r.run();
    assert_eq!(r.status(), Status::Completed, "{:?}", r.error());
    let ran: Vec<String> = ["init", "call", "big", "small"].iter().filter(|x| r.ran(x)).map(|x| x.to_string()).collect();
    let result = num(&r, "result");
    (p.clone(), ran, result)
}

#[test]
fn a_sequence_with_a_call_and_a_decision_takes_the_guarded_branch() {
    let (_, ran, result) = run_a("5");
    assert_eq!(ran, ["init", "call", "big"]);
    assert_eq!(result, "15", "5 * 3 = 15 > 10");
    let (_, ran, result) = run_a("2");
    assert_eq!(ran, ["init", "call", "small"], "6 is not > 10, so the else branch");
    assert_eq!(result, "2");
}

#[test]
fn fork_and_join_run_branches_in_document_order_and_join_waits_for_all() {
    let p = program(
        &[(
            "F",
            "variables:
  - {name: log, value: \"0\"}
  - {name: a, value: \"0\"}
  - {name: b, value: \"0\"}
subActions:
  - {name: first, kind: AssignmentAction, assign: \"log := 1\"}
  - {name: left, kind: AssignmentAction, assign: \"a := log + 1\"}
  - {name: right, kind: AssignmentAction, assign: \"b := log + 2\"}
  - {name: last, kind: AssignmentAction, assign: \"log := a + b\"}
controlNodes:
  - {name: f, kind: ForkNode}
  - {name: j, kind: JoinNode}
successionConnections:
  - {after: first, before: f}
  - {after: f, before: left}
  - {after: f, before: right}
  - {after: left, before: j}
  - {after: right, before: j}
  - {after: j, before: last}
",
        )],
        "F",
    );
    let mut r = ActRun::new(&p, ActOptions::default());
    r.run();
    assert_eq!(r.status(), Status::Completed);
    assert_eq!(num(&r, "log"), "5", "a = 2, b = 3, then log := a + b");
    let order: Vec<_> = r.trace().iter().map(|s| s.node.clone()).collect();
    assert_eq!(order, ["first", "f", "left", "right", "j", "last"]);
}

#[test]
fn a_merge_forwards_whichever_branch_arrived() {
    let p = program(
        &[(
            "M",
            "variables:
  - {name: v, value: \"1\"}
  - {name: out, value: \"0\"}
subActions:
  - {name: x, kind: AssignmentAction, assign: \"out := out + 1\"}
  - {name: y, kind: AssignmentAction, assign: \"out := out + 10\"}
  - {name: z, kind: AssignmentAction, assign: \"out := out + 100\"}
controlNodes:
  - {name: d, kind: DecisionNode}
  - {name: m, kind: MergeNode}
successionConnections:
  - {after: start, before: d}
  - {after: d, before: x, guard: \"v == 1\"}
  - {after: d, before: y, guard: else}
  - {after: x, before: m}
  - {after: y, before: m}
  - {after: m, before: z}
",
        )],
        "M",
    );
    let mut r = ActRun::new(&p, ActOptions::default());
    r.run();
    assert_eq!(num(&r, "out"), "101");
    assert!(!r.ran("y"));
}

#[test]
fn if_and_loops_run_their_blocks() {
    let p = program(
        &[(
            "L",
            "variables:
  - {name: i, value: \"0\"}
  - {name: u, value: \"0\"}
  - {name: tag, value: \"0\"}
subActions:
  - name: w
    kind: LoopAction
    loopKind: while
    condition: \"i < 3\"
    body:
      - {name: inc, kind: AssignmentAction, assign: \"i := i + 1\"}
  - name: once
    kind: LoopAction
    loopKind: until
    condition: \"u >= 1\"
    body:
      - {name: bump, kind: AssignmentAction, assign: \"u := u + 1\"}
  - name: choose
    kind: IfAction
    condition: \"i == 3\"
    then:
      - {name: yes, kind: AssignmentAction, assign: \"tag := 10\"}
    else:
      - {name: no, kind: AssignmentAction, assign: \"tag := 20\"}
successionConnections:
  - {after: w, before: once}
  - {after: once, before: choose}
",
        )],
        "L",
    );
    let mut r = ActRun::new(&p, ActOptions::default());
    r.run();
    assert_eq!((num(&r, "i"), num(&r, "u"), num(&r, "tag")), ("3".into(), "1".into(), "10".into()), "until runs its body at least once");
}

#[test]
fn a_runaway_loop_is_an_error() {
    let p = program(&[("R", "variables:\n  - {name: i, value: \"0\"}\nsubActions:\n  - name: w\n    kind: LoopAction\n    loopKind: while\n    condition: \"true\"\n    body:\n      - {name: inc, kind: AssignmentAction, assign: \"i := i + 1\"}\n")], "R");
    let mut r = ActRun::new(&p, ActOptions::default());
    r.run();
    assert_eq!(r.status(), Status::Error);
    assert!(r.error().is_some_and(|e| e.contains("iterations")), "{:?}", r.error());
}

#[test]
fn terminate_ends_the_whole_action() {
    let p = program(&[("T", "variables:\n  - {name: v, value: \"0\"}\nsubActions:\n  - {name: a, kind: AssignmentAction, assign: \"v := 1\"}\n  - {name: stop, kind: TerminateAction}\n  - {name: after, kind: AssignmentAction, assign: \"v := 2\"}\n  - {name: side, kind: AssignmentAction, assign: \"v := 3\"}\nsuccessionConnections:\n  - {after: a, before: stop}\n  - {after: stop, before: after}\n")], "T");
    let mut r = ActRun::new(&p, ActOptions::default());
    r.run();
    assert_eq!(r.status(), Status::Terminated);
    assert!(!r.ran("after"));
    assert_eq!(num(&r, "v"), "1", "`side` has no incoming edge and would have run second; terminate wins only for nodes after it in document order");
}

#[test]
fn accept_waits_for_its_event_and_binds_the_payload() {
    let p = program(
        &[(
            "W",
            "variables:\n  - {name: got, value: \"0\"}\nsubActions:\n  - {name: wait, kind: AcceptAction, payload: Pkg::Go, as: g}\n  - {name: take, kind: AssignmentAction, assign: \"got := g.v\"}\nsuccessionConnections:\n  - {after: wait, before: take}\n",
        )],
        "W",
    );
    let mut r = ActRun::new(&p, ActOptions::default());
    r.run();
    assert_eq!(r.status(), Status::Waiting);
    r.send("Pkg::Other", &[]);
    assert_eq!(r.status(), Status::Waiting, "an unrelated event is ignored");
    r.send("Pkg::Go", &[("v", val("7"))]);
    assert_eq!(r.status(), Status::Completed);
    assert_eq!(num(&r, "got"), "7");
}

#[test]
fn a_timed_accept_completes_on_the_virtual_clock() {
    let p = program(&[("T", "variables:\n  - {name: v, value: \"0\"}\nsubActions:\n  - {name: wait, kind: AcceptAction, after: \"2 s\"}\n  - {name: go, kind: AssignmentAction, assign: \"v := 1\"}\nsuccessionConnections:\n  - {after: wait, before: go}\n")], "T");
    let mut r = ActRun::new(&p, ActOptions::default());
    r.run();
    r.advance(1.9);
    assert_eq!(r.status(), Status::Waiting);
    r.advance(0.2);
    assert_eq!(r.status(), Status::Completed);
    assert_eq!(num(&r, "v"), "1");
    assert!((r.time() - 2.1).abs() < 1e-9);
}

#[test]
fn send_records_the_event_and_its_payload() {
    let p = program(&[("S", "variables:\n  - {name: v, value: \"4\"}\nsubActions:\n  - name: out\n    kind: SendAction\n    payload: Pkg::Level\n    via: p\n    parameters:\n      - {name: value, value: \"v * 2\"}\n")], "S");
    let mut r = ActRun::new(&p, ActOptions::default());
    r.run();
    assert_eq!(r.sent().len(), 1);
    assert_eq!(r.sent()[0].event, "p/Pkg::Level");
    assert_eq!(r.sent()[0].args, [("value".to_string(), "8".to_string())]);
}

#[test]
fn object_flow_copies_a_pin_when_its_node_completes() {
    let p = program(
        &[
            ("B", B),
            (
                "O",
                "parameters:\n  - {name: out, direction: out}\nvariables:\n  - {name: seed, value: \"2\"}\nsubActions:\n  - {name: p1, kind: PerformAction, typedBy: B, parameters: [{name: x, value: seed}]}\n  - {name: p2, kind: PerformAction, typedBy: B}\nsuccessionConnections:\n  - {after: p1, before: p2}\nflowConnections:\n  - {from: p1.y, to: p2.x}\nbindingConnections:\n  - {left: p2.y, right: out}\n",
            ),
        ],
        "O",
    );
    let mut r = ActRun::new(&p, ActOptions::default());
    r.run();
    assert_eq!(r.status(), Status::Completed, "{:?}", r.error());
    assert_eq!(num(&r, "out"), "18", "2 * 3 flows into the second call: 6 * 3");
}

#[test]
fn errors_stop_the_run_and_are_recorded_in_the_trace() {
    let p = program(&[("E", "parameters:\n  - {name: n, direction: in}\nvariables:\n  - {name: v, value: \"0\"}\nsubActions:\n  - {name: a, kind: AssignmentAction, assign: \"v := n\"}\n")], "E");
    let mut r = ActRun::new(&p, ActOptions::default());
    r.run();
    assert_eq!(r.status(), Status::Error);
    assert!(r.error().is_some_and(|e| e.contains("'n'")), "an input with no value: {:?}", r.error());
    assert_eq!(r.trace().last().unwrap().node, "error");
    // `x` is an out parameter without a default: untyped statically, so only the run finds the dimension error (`t := 5` is E935)
    let q = program(&[("E2", "parameters:\n  - {name: x, direction: out}\nvariables:\n  - {name: t, value: \"1 s\"}\nsubActions:\n  - {name: s1, kind: AssignmentAction, assign: \"x := 5\"}\n  - {name: a, kind: AssignmentAction, assign: \"t := x\"}\nsuccessionConnections:\n  - {after: s1, before: a}\n")], "E2");
    let mut r = ActRun::new(&q, ActOptions::default());
    r.run();
    assert_eq!(r.status(), Status::Error, "{:?}", r.error());
}

#[test]
fn a_guard_that_selects_no_edge_is_a_note_and_an_error_in_strict_mode() {
    let src = "variables:\n  - {name: v, value: \"0\"}\nsubActions:\n  - {name: a, kind: AssignmentAction, assign: \"v := 1\"}\n  - {name: b, kind: AssignmentAction, assign: \"v := 2\"}\nsuccessionConnections:\n  - {after: a, before: b, guard: \"v == 99\"}\n";
    let p = program(&[("G", src)], "G");
    let mut r = ActRun::new(&p, ActOptions::default());
    r.run();
    assert_eq!(r.status(), Status::Completed);
    assert!(!r.ran("b"));
    assert!(r.trace().iter().any(|s| s.note.as_deref().is_some_and(|n| n.contains("no outgoing"))));
    let mut s = ActRun::new(&p, ActOptions { strict: true, ..ActOptions::default() });
    s.run();
    assert_eq!(s.status(), Status::Error);
}

#[test]
fn a_called_action_that_waits_is_an_error() {
    let p = program(&[("W", "variables:\n  - {name: v, value: \"0\"}\nsubActions:\n  - {name: wait, kind: AcceptAction, payload: Pkg::Go}\n"), ("C", "variables:\n  - {name: v, value: \"0\"}\nsubActions:\n  - {name: c, kind: PerformAction, typedBy: W}\n")], "C");
    let mut r = ActRun::new(&p, ActOptions::default());
    r.run();
    assert_eq!(r.status(), Status::Error);
    assert!(r.error().is_some_and(|e| e.contains("waits")), "{:?}", r.error());
}

#[test]
fn a_rerun_gives_an_identical_trace() {
    let p = program(&[("A", A), ("B", B)], "A");
    let go = || {
        let mut r = ActRun::new(&p, ActOptions::default());
        r.set_input("n", val("5")).unwrap();
        r.run();
        serde_json::to_string(r.trace()).unwrap()
    };
    assert_eq!(go(), go());
}

// ── review fixes (GH #276) ───────────────────────────────────────────────────

#[test]
fn a_loop_head_named_after_start_fires_even_though_it_has_a_back_edge() {
    let p = program(
        &[(
            "H",
            "variables:
  - {name: i, value: \"0\"}
  - {name: done, value: \"0\"}
subActions:
  - {name: inc, kind: AssignmentAction, assign: \"i := i + 1\"}
  - {name: fin, kind: AssignmentAction, assign: \"done := 1\"}
controlNodes:
  - {name: m, kind: MergeNode}
  - {name: d, kind: DecisionNode}
successionConnections:
  - {after: start, before: m}
  - {after: m, before: inc}
  - {after: inc, before: d}
  - {after: d, before: m, guard: \"i < 3\"}
  - {after: d, before: fin, guard: else}
",
        )],
        "H",
    );
    let mut r = ActRun::new(&p, ActOptions::default());
    r.run();
    assert_eq!(r.status(), Status::Completed, "{:?}", r.error());
    assert_eq!((num(&r, "i"), num(&r, "done")), ("3".into(), "1".into()));
}

#[test]
fn an_until_loop_runs_its_body_before_testing_the_condition() {
    let p = program(
        &[(
            "U",
            "parameters:
  - {name: r, direction: out}
variables:
  - {name: i, value: \"0\"}
subActions:
  - name: u
    kind: LoopAction
    loopKind: until
    condition: \"r >= 3\"
    body:
      - {name: inc, kind: AssignmentAction, assign: \"i := i + 1\"}
      - {name: set, kind: AssignmentAction, assign: \"r := i\"}
",
        )],
        "U",
    );
    let mut r = ActRun::new(&p, ActOptions::default());
    r.run();
    assert_eq!(r.status(), Status::Completed, "{:?}", r.error());
    assert_eq!((num(&r, "i"), num(&r, "r")), ("3".into(), "3".into()));
}

#[test]
fn terminate_inside_a_loop_body_stops_the_loop() {
    let p = program(&[("T", "variables:\n  - {name: i, value: \"0\"}\nsubActions:\n  - name: w\n    kind: LoopAction\n    loopKind: while\n    condition: \"i < 5\"\n    body:\n      - {name: inc, kind: AssignmentAction, assign: \"i := i + 1\"}\n      - {name: stop, kind: TerminateAction}\n")], "T");
    let mut r = ActRun::new(&p, ActOptions::default());
    r.run();
    assert_eq!(r.status(), Status::Terminated);
    assert_eq!(num(&r, "i"), "1");
}

#[test]
fn tokens_left_on_successions_are_a_deadlock_not_a_completion() {
    let p = program(
        &[(
            "D",
            "variables:\n  - {name: v, value: \"1\"}\nsubActions:\n  - {name: a, kind: Action}\n  - {name: b, kind: Action}\n  - {name: c, kind: Action}\ncontrolNodes:\n  - {name: d, kind: DecisionNode}\nsuccessionConnections:\n  - {after: d, before: a, guard: \"v == 1\"}\n  - {after: d, before: b, guard: else}\n  - {after: a, before: c}\n  - {after: b, before: c}\n",
        )],
        "D",
    );
    let mut r = ActRun::new(&p, ActOptions::default());
    r.run();
    assert_eq!(r.status(), Status::Deadlocked, "c needs a token from both a and b");
    assert!(r.trace().iter().any(|s| s.note.as_deref().is_some_and(|n| n.contains("deadlock"))));
}

#[test]
fn a_binding_from_a_parameter_to_a_callee_input_feeds_the_callee() {
    let p = program(
        &[
            ("B", B),
            ("I", "parameters:\n  - {name: inp, direction: in}\n  - {name: out, direction: out}\nvariables:\n  - {name: v, value: \"0\"}\nsubActions:\n  - {name: call, kind: PerformAction, typedBy: B}\nbindingConnections:\n  - {left: inp, right: call.x}\n  - {left: call.y, right: out}\n"),
        ],
        "I",
    );
    let mut r = ActRun::new(&p, ActOptions::default());
    r.set_input("inp", val("4")).unwrap();
    r.run();
    assert_eq!(r.status(), Status::Completed, "{:?}", r.error());
    assert_eq!(num(&r, "out"), "12");
}

#[test]
fn nested_conditions_have_their_own_coverage_keys() {
    let p = program(
        &[(
            "N",
            "variables:\n  - {name: v, value: \"1\"}\nsubActions:\n  - name: outer\n    kind: IfAction\n    condition: \"v == 1\"\n    then:\n      - name: inner\n        kind: IfAction\n        condition: \"v > 5\"\n        then:\n          - {name: x, kind: AssignmentAction, assign: \"v := 2\"}\n",
        )],
        "N",
    );
    let mut r = ActRun::new(&p, ActOptions::default());
    r.run();
    let cov = r.coverage();
    assert_eq!(cov.conds.len(), 2, "outer and inner conditions are separate keys: {:?}", cov.conds.keys().collect::<Vec<_>>());
}

#[test]
fn a_performed_actions_stale_outputs_do_not_survive_a_second_call() {
    let p = program(
        &[
            ("B2", "parameters:\n  - {name: x, direction: in}\n  - {name: y, direction: out}\nvariables:\n  - {name: v, value: \"0\"}\nsubActions:\n  - name: m\n    kind: IfAction\n    condition: \"x > 0\"\n    then:\n      - {name: s, kind: AssignmentAction, assign: \"y := x\"}\n"),
            ("S", "variables:\n  - {name: r, value: \"0\"}\nsubActions:\n  - {name: c, kind: PerformAction, typedBy: B2, parameters: [{name: x, value: \"r + 1\"}]}\n  - {name: d, kind: PerformAction, typedBy: B2, parameters: [{name: x, value: \"0 - 1\"}]}\n  - {name: use, kind: AssignmentAction, assign: \"r := d.y\"}\nsuccessionConnections:\n  - {after: c, before: d}\n  - {after: d, before: use}\n"),
        ],
        "S",
    );
    let mut r = ActRun::new(&p, ActOptions::default());
    r.run();
    assert_eq!(r.status(), Status::Error, "d produced no y; the stale c.y must not be read");
}
