//! TC-TRS-SMEXEC-001 / REQ-TRS-SMEXEC-001 (GH #279): the state-machine interpreter.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::expr::{parse_value, Value};
use syscribe_model::ir::{lower, Machine};
use syscribe_model::sm_run::{Options, Run};
use syscribe_model::walker::walk_model;

fn machine(yaml: &str) -> Machine {
    static N: AtomicU64 = AtomicU64::new(0);
    let root: PathBuf = std::env::temp_dir().join(format!("syscribe-smrun-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(root.join("M.md"), format!("---\ntype: StateDef\nname: M\n{yaml}---\n")).unwrap();
    let elems = walk_model(&root).unwrap();
    lower(&elems, "M").unwrap_or_else(|d| panic!("lowering failed: {d:?}"))
}

fn val(s: &str) -> Value {
    parse_value(s).unwrap()
}

fn num(r: &Run, name: &str) -> String {
    syscribe_model::expr::format_value(&r.var(name).unwrap())
}

const TWO: &str = "variables:
  - {name: count, value: \"0\"}
subStates:
  - name: A
    isInitial: true
    entryAction: {assign: \"count := count + 1\"}
    transitions:
      - {target: B, accept: Pkg::Go, guard: \"count < 3\"}
  - name: B
    transitions:
      - {target: A, accept: Pkg::Back}
";

#[test]
fn start_enters_the_initial_state_and_runs_its_entry_actions() {
    let m = machine(TWO);
    let r = Run::new(&m, Options::default());
    assert_eq!(r.active(), ["A"]);
    assert_eq!(num(&r, "count"), "1");
    assert!(r.error().is_none());
}

#[test]
fn a_transition_runs_exit_then_effect_then_entry() {
    let m = machine(
        "subStates:
  - name: A
    isInitial: true
    exitAction: Pkg::XA
    transitions:
      - {target: B, accept: Pkg::Go, effect: Pkg::T}
  - name: B
    entryAction: Pkg::EB
",
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Go", &[]);
    assert_eq!(r.active(), ["B"]);
    let step = r.trace().iter().find(|s| s.transition.is_some()).unwrap();
    assert_eq!(step.actions, ["behavior Pkg::XA", "behavior Pkg::T", "behavior Pkg::EB"]);
    assert_eq!(step.exited, ["A"]);
    assert_eq!(step.entered, ["B"]);
}

#[test]
fn a_false_guard_leaves_the_event_unhandled() {
    let m = machine(TWO);
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Go", &[]); // count 1 → A→B, B→A on Back runs entry → count 2
    r.send("Pkg::Back", &[]);
    r.send("Pkg::Go", &[]);
    r.send("Pkg::Back", &[]); // count 3, guard count<3 now false
    assert_eq!(r.active(), ["A"]);
    r.send("Pkg::Go", &[]);
    assert_eq!(r.active(), ["A"], "guard false: not taken");
    assert!(r.trace().last().unwrap().note.as_deref().is_some_and(|n| n.contains("ignored")));
    assert!(r.error().is_none());
    let mut s = Run::new(&m, Options { strict: true, ..Options::default() });
    s.send("Pkg::Nothing", &[]);
    assert!(s.error().is_some_and(|e| e.contains("Pkg::Nothing")), "strict: unhandled event is an error");
}

#[test]
fn a_deeper_state_preempts_its_ancestor() {
    let m = machine(
        "subStates:
  - name: P
    isInitial: true
    transitions:
      - {target: Out, accept: Pkg::Go}
    subStates:
      - name: C
        isInitial: true
        transitions:
          - {target: D, accept: Pkg::Go}
      - name: D
  - name: Out
",
    );
    let mut r = Run::new(&m, Options::default());
    assert_eq!(r.active(), ["P", "P.C"]);
    r.send("Pkg::Go", &[]);
    assert_eq!(r.active(), ["P", "P.D"], "the child's transition wins over the parent's");
    r.send("Pkg::Go", &[]);
    assert_eq!(r.active(), ["Out"], "then the parent's applies and the child is exited");
}

#[test]
fn after_timers_fire_on_the_virtual_clock_and_are_cancelled_on_exit() {
    let m = machine(
        "subStates:
  - name: A
    isInitial: true
    transitions:
      - {target: B, accept: {after: \"2 s\"}}
      - {target: C, accept: Pkg::Leave}
  - name: B
  - name: C
    transitions:
      - {target: A, accept: Pkg::Again}
",
    );
    let mut r = Run::new(&m, Options::default());
    r.advance(1.0);
    assert_eq!(r.active(), ["A"]);
    r.advance(0.999);
    assert_eq!(r.active(), ["A"]);
    r.advance(0.002);
    assert_eq!(r.active(), ["B"]);
    // leaving before the timer is due cancels it
    let mut r = Run::new(&m, Options::default());
    r.advance(1.0);
    r.send("Pkg::Leave", &[]);
    r.advance(10.0);
    assert_eq!(r.active(), ["C"], "the cancelled timer never fires");
    // re-entry re-arms it from the new entry time
    r.send("Pkg::Again", &[]);
    r.advance(1.9);
    assert_eq!(r.active(), ["A"]);
    r.advance(0.2);
    assert_eq!(r.active(), ["B"]);
    assert!((r.time() - 13.1).abs() < 1e-9);
}

#[test]
fn a_zero_delay_fires_on_entry() {
    let m = machine("subStates:\n  - name: A\n    isInitial: true\n    transitions:\n      - {target: B, accept: {after: \"0 s\"}}\n  - name: B\n");
    let r = Run::new(&m, Options::default());
    assert_eq!(r.active(), ["B"]);
}

const CHANGE: &str = "variables:
  - {name: level, value: \"0\"}
subStates:
  - name: A
    isInitial: true
    transitions:
      - {target: B, accept: {when: \"level > 2\"}}
  - name: B
    transitions:
      - {target: A, accept: Pkg::Back}
";

#[test]
fn a_when_trigger_fires_on_false_to_true() {
    let m = machine(CHANGE);
    let mut r = Run::new(&m, Options::default());
    r.set("level", val("1"));
    r.set("level", val("2"));
    assert_eq!(r.active(), ["A"]);
    r.set("level", val("3"));
    assert_eq!(r.active(), ["B"]);
}

#[test]
fn a_when_already_true_on_entry_fires_at_once_and_only_once_per_arming() {
    let m = machine(&CHANGE.replace("value: \"0\"", "value: \"5\""));
    let r = Run::new(&m, Options::default());
    assert_eq!(r.active(), ["B"], "true when armed: the signal is sent at once");
    // re-entering A re-arms; level is still 5 so it fires again, but only once each arming
    let mut r = r;
    r.send("Pkg::Back", &[]);
    assert_eq!(r.active(), ["B"]);
    assert_eq!(r.trace().iter().filter(|s| s.cause.starts_with("change")).count(), 2);
}

#[test]
fn a_when_signal_is_consumed_even_if_its_guard_fails() {
    let m = machine(
        "variables:
  - {name: level, value: \"0\"}
  - {name: armed, value: \"false\"}
subStates:
  - name: A
    isInitial: true
    transitions:
      - {target: B, accept: {when: \"level > 2\"}, guard: \"armed\"}
  - name: B
",
    );
    let mut r = Run::new(&m, Options::default());
    r.set("level", val("3"));
    r.set("armed", val("true"));
    assert_eq!(r.active(), ["A"], "the signal was spent while the guard was false and is not re-delivered");
    r.set("level", val("0"));
    r.set("level", val("3"));
    assert_eq!(r.active(), ["A"], "armed stays set only on re-entry; the observation is still spent");
}

#[test]
fn done_triggers_follow_final_states_and_internal_transitions_do_not_exit() {
    let m = machine(
        "variables:
  - {name: pings, value: \"0\"}
subStates:
  - name: W
    isInitial: true
    exitAction: Pkg::XW
    entryAction: Pkg::EW
    transitions:
      - {internal: true, accept: Pkg::Ping, assign: \"pings := pings + 1\"}
      - {target: End, accept: {done: W}}
    subStates:
      - name: Work
        isInitial: true
        transitions:
          - {target: Fin, accept: Pkg::Finish}
      - name: Fin
        isFinal: true
  - name: End
",
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Ping", &[]);
    assert_eq!(num(&r, "pings"), "1");
    let step = r.trace().last().unwrap();
    assert!(step.exited.is_empty() && step.entered.is_empty(), "internal: no exit, no entry");
    r.send("Pkg::Finish", &[]);
    assert_eq!(r.active(), ["End"], "reaching the final child completes W");
}

#[test]
fn parallel_regions_react_together() {
    let m = machine(
        "isParallel: true
variables:
  - {name: a, value: \"0\"}
  - {name: b, value: \"0\"}
subStates:
  - name: R1
    subStates:
      - name: X1
        isInitial: true
        transitions:
          - {target: X2, accept: Pkg::Go, assign: \"a := 1\"}
      - name: X2
  - name: R2
    subStates:
      - name: Y1
        isInitial: true
        transitions:
          - {target: Y2, accept: Pkg::Go, assign: \"b := 1\"}
      - name: Y2
",
    );
    let mut r = Run::new(&m, Options::default());
    assert_eq!(r.active(), ["R1", "R1.X1", "R2", "R2.Y1"]);
    r.send("Pkg::Go", &[]);
    assert_eq!(r.active(), ["R1", "R1.X2", "R2", "R2.Y2"]);
    assert_eq!((num(&r, "a"), num(&r, "b")), ("1".into(), "1".into()));
    assert_eq!(r.trace().iter().filter(|s| s.transition.is_some()).count(), 1, "one microstep took both transitions");
}

#[test]
fn payloads_bind_to_guards_and_assignments() {
    let m = machine(
        "variables:
  - {name: total, value: \"0\"}
subStates:
  - name: A
    isInitial: true
    transitions:
      - {internal: true, accept: {payload: Pkg::Add, as: ev}, guard: \"ev.n > 1\", assign: \"total := total + ev.n\"}
",
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Add", &[("n", val("1"))]);
    assert_eq!(num(&r, "total"), "0");
    r.send("Pkg::Add", &[("n", val("5"))]);
    assert_eq!(num(&r, "total"), "5");
    r.send("Pkg::Add", &[]);
    assert!(r.error().is_some_and(|e| e.contains("ev.n")), "missing payload field stops the run: {:?}", r.error());
}

#[test]
fn type_and_dimension_errors_stop_the_run() {
    let m = machine(
        "variables:
  - {name: t, value: \"1 s\"}
subStates:
  - name: A
    isInitial: true
    transitions:
      - {internal: true, accept: {payload: Pkg::Bad, as: m}, assign: \"t := m.v\"}
      - {internal: true, accept: Pkg::Ok, assign: \"t := t + 2 s\"}
",
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Ok", &[]);
    assert_eq!(num(&r, "t"), "3 s");
    r.send("Pkg::Bad", &[("v", val("5"))]);
    assert!(r.error().is_some_and(|e| e.contains("t")), "{:?}", r.error());
    let before = r.trace().len();
    r.send("Pkg::Ok", &[]);
    assert_eq!(r.trace().len(), before, "nothing runs after an error");
}

#[test]
fn a_livelock_hits_the_microstep_bound() {
    let m = machine("subStates:\n  - name: A\n    isInitial: true\n    transitions:\n      - {target: B}\n  - name: B\n    transitions:\n      - {target: A}\n");
    let r = Run::new(&m, Options { max_microsteps: 50, ..Options::default() });
    assert!(r.error().is_some_and(|e| e.contains("microstep")), "{:?}", r.error());
}

#[test]
fn a_rerun_gives_an_identical_trace() {
    let m = machine(TWO);
    let run = || {
        let mut r = Run::new(&m, Options::default());
        r.send("Pkg::Go", &[]);
        r.advance(1.5);
        r.send("Pkg::Back", &[]);
        serde_json::to_string(r.trace()).unwrap()
    };
    assert_eq!(run(), run());
}

// ── review fixes (GH #279) ───────────────────────────────────────────────────

#[test]
fn a_descendant_transition_in_one_region_preempts_the_ancestors_when_another_region_has_none() {
    let m = machine(
        "subStates:
  - name: P
    isInitial: true
    isParallel: true
    transitions:
      - {target: Z, accept: Pkg::Go}
    subStates:
      - name: R1
        subStates:
          - {name: a, isInitial: true}
      - name: R2
        subStates:
          - {name: b, isInitial: true, transitions: [{target: c, accept: Pkg::Go}]}
          - {name: c}
  - name: Z
",
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Go", &[]);
    assert_eq!(r.active(), ["P", "P.R1", "P.R1.a", "P.R2", "P.R2.c"], "R2.b → R2.c wins; P is not left");
}

#[test]
fn repeated_assignments_to_one_variable_run_in_order_and_strings_may_contain_semicolons() {
    let m = machine(
        "variables:
  - {name: x, value: \"0\"}
  - {name: s, value: \"'z'\"}
subStates:
  - name: A
    isInitial: true
    entryAction: {assign: \"x := 1; x := x + 1; s := 'a;b'\"}
",
    );
    let r = Run::new(&m, Options::default());
    assert_eq!(num(&r, "x"), "2");
    assert_eq!(r.var("s"), Some(Value::Str("a;b".into())));
}

#[test]
fn a_nested_compound_without_an_initial_child_starts_in_its_first() {
    let m = machine("subStates:\n  - name: A\n    isInitial: true\n    transitions:\n      - {target: C, accept: Pkg::Go}\n  - name: C\n    subStates:\n      - {name: x}\n      - {name: y}\n");
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Go", &[]);
    assert_eq!(r.active(), ["C", "C.x"]);
}

#[test]
fn a_when_on_a_composite_is_armed_after_the_whole_entry_set_has_run() {
    let m = machine(
        "variables:
  - {name: v, value: \"0\"}
subStates:
  - name: A
    isInitial: true
    transitions:
      - {target: C, accept: Pkg::Go}
  - name: C
    transitions:
      - {target: Out, accept: {when: \"v == 1\"}}
    subStates:
      - name: c1
        isInitial: true
        entryAction: {assign: \"v := 1\"}
  - name: Out
",
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Go", &[]);
    assert_eq!(r.active(), ["Out"], "v is 1 once c1's entry action has run, so the observation starts true");
}

#[test]
fn a_machine_level_parallel_terminates_when_every_region_is_final() {
    let m = machine(
        "isParallel: true
subStates:
  - name: R1
    subStates:
      - {name: a, isInitial: true, transitions: [{target: af, accept: Pkg::Go}]}
      - {name: af, isFinal: true}
  - name: R2
    subStates:
      - {name: b, isInitial: true, transitions: [{target: bf, accept: Pkg::Go}]}
      - {name: bf, isFinal: true}
",
    );
    let mut r = Run::new(&m, Options::default());
    assert!(!r.terminated());
    r.send("Pkg::Go", &[]);
    assert!(r.terminated());
}

#[test]
fn a_run_error_is_recorded_in_the_trace() {
    // the payload field has no static type, so only the run can find the dimension error (a literal `1 s + 1` is E935)
    let m = machine("subStates:\n  - name: A\n    isInitial: true\n    transitions:\n      - {internal: true, accept: {payload: Pkg::E, as: m}, guard: \"m.level + 1 s > 0 s\"}\n");
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::E", &[("level", val("1"))]);
    let last = r.trace().last().unwrap();
    assert_eq!(last.cause, "error");
    assert!(last.note.as_deref().is_some_and(|n| n.contains("guard")), "{last:?}");
}

#[test]
fn strict_mode_reports_any_conflicting_pair() {
    let m = machine("subStates:\n  - name: A\n    isInitial: true\n    transitions:\n      - {target: B, accept: Pkg::Go}\n      - {target: B, accept: Pkg::Go}\n      - {target: C, accept: Pkg::Go}\n  - {name: B}\n  - {name: C}\n");
    let r = {
        let mut r = Run::new(&m, Options { strict: true, ..Options::default() });
        r.send("Pkg::Go", &[]);
        r
    };
    assert!(r.error().is_some_and(|e| e.contains("t0") && e.contains("t2")), "{:?}", r.error());
}
