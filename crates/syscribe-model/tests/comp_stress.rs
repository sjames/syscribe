//! Stress tests for activities driven by state machines, and for several such parts running as one system.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::expr::{format_value, parse_value};
use syscribe_model::ir::{lower, Machine};
use syscribe_model::sm_run::{Options, Run};
use syscribe_model::system_run::{System, SystemModel};
use syscribe_model::walker::walk_model;

fn tmp(tag: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let d = std::env::temp_dir().join(format!("syscribe-comp-{tag}-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    d
}

fn machine(m_body: &str, actions: &[(&str, &str)]) -> Machine {
    let d = tmp("sm");
    std::fs::write(d.join("M.md"), format!("---\ntype: StateDef\nname: M\n{m_body}---\n")).unwrap();
    for (n, b) in actions {
        std::fs::write(d.join(format!("{n}.md")), format!("---\ntype: ActionDef\nname: {n}\n{b}---\n")).unwrap();
    }
    lower(&walk_model(&d).unwrap(), "M").unwrap_or_else(|e| panic!("{e:?}"))
}

fn num(r: &Run, n: &str) -> String {
    format_value(&r.var(n).unwrap())
}

/// Appends a digit to the machine's `log` variable.
fn digit(name: &str, d: u32) -> (String, String) {
    (
        name.to_string(),
        format!("parameters:\n  - {{name: log, direction: inout}}\nsubActions:\n  - {{name: a, kind: AssignmentAction, assign: \"log := log * 10 + {d}\"}}\n"),
    )
}

fn with_digits(body: &str, ds: &[(&str, u32)]) -> Machine {
    let acts: Vec<(String, String)> = ds.iter().map(|(n, d)| digit(n, *d)).collect();
    let refs: Vec<(&str, &str)> = acts.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    machine(body, &refs)
}

// ── state machine + activity ─────────────────────────────────────────────────

#[test]
fn entry_and_exit_actions_across_a_hierarchy_run_their_activities_in_order() {
    let m = with_digits(
        "variables:
  - {name: log, value: \"0\"}
subStates:
  - name: P
    isInitial: true
    entryAction: D1
    exitAction: D2
    transitions: [{target: Q, accept: Pkg::Go, effect: D3}]
    subStates:
      - {name: C, isInitial: true, entryAction: D4, exitAction: D5}
  - name: Q
    entryAction: D6
    subStates:
      - {name: E, isInitial: true, entryAction: D7}
",
        &[("D1", 1), ("D2", 2), ("D3", 3), ("D4", 4), ("D5", 5), ("D6", 6), ("D7", 7)],
    );
    let mut r = Run::new(&m, Options::default());
    assert_eq!(num(&r, "log"), "14", "P then C");
    r.send("Pkg::Go", &[]);
    assert!(r.error().is_none(), "{:?}", r.error());
    assert_eq!(num(&r, "log"), "1452367", "C exit, P exit, effect, Q entry, E entry appended in order");
}

#[test]
fn the_order_of_entry_exit_and_effect_activities_is_exit_effect_entry() {
    let m = with_digits(
        "variables:
  - {name: log, value: \"0\"}
subStates:
  - name: P
    isInitial: true
    exitAction: D2
    transitions: [{target: Q, accept: Pkg::Go, effect: D3}]
    subStates:
      - {name: C, isInitial: true, exitAction: D5}
  - name: Q
    entryAction: D6
    subStates:
      - {name: E, isInitial: true, entryAction: D7}
",
        &[("D2", 2), ("D3", 3), ("D5", 5), ("D6", 6), ("D7", 7)],
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Go", &[]);
    assert!(r.error().is_none(), "{:?}", r.error());
    assert_eq!(num(&r, "log"), "52367", "C exit, P exit, effect, Q entry, E entry");
}

#[test]
fn parallel_regions_that_both_run_activities_on_one_variable_do_so_in_region_order() {
    let m = with_digits(
        "variables:
  - {name: log, value: \"0\"}
subStates:
  - name: P
    isInitial: true
    isParallel: true
    subStates:
      - name: R1
        subStates: [{name: a, isInitial: true, entryAction: D1}]
      - name: R2
        subStates: [{name: b, isInitial: true, entryAction: D2}]
",
        &[("D1", 1), ("D2", 2)],
    );
    let r = Run::new(&m, Options::default());
    assert!(r.error().is_none(), "{:?}", r.error());
    assert_eq!(num(&r, "log"), "12");
}

#[test]
fn an_activity_that_changes_a_watched_variable_fires_the_when_trigger() {
    let m = machine(
        "variables:
  - {name: level, value: \"0\"}
subStates:
  - name: Idle
    isInitial: true
    transitions:
      - {target: Pumping, accept: Pkg::Fill}
  - name: Pumping
    entryAction: Raise
    transitions:
      - {target: Full, accept: {when: \"level >= 10\"}}
  - name: Full
",
        &[("Raise", "parameters:\n  - {name: level, direction: inout}\nsubActions:\n  - {name: a, kind: AssignmentAction, assign: \"level := level + 10\"}\n")],
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Fill", &[]);
    assert_eq!(r.active(), ["Full"], "the change made by the entry activity is seen by the `when` armed after entry");
}

#[test]
fn an_activitys_local_variables_do_not_leak_into_the_machine() {
    let m = machine(
        "variables:
  - {name: secret, value: \"7\"}
  - {name: y, value: \"0\"}
subStates:
  - name: A
    isInitial: true
    entryAction: Act
",
        &[("Act", "parameters:\n  - {name: y, direction: out}\nvariables:\n  - {name: secret, value: \"99\"}\nsubActions:\n  - {name: a, kind: AssignmentAction, assign: \"y := secret\"}\n")],
    );
    let r = Run::new(&m, Options::default());
    assert_eq!((num(&r, "secret"), num(&r, "y")), ("7".into(), "99".into()));
}

#[test]
fn an_activity_output_with_no_matching_machine_variable_is_silently_dropped() {
    // Characterisation: a typo in an `out` parameter name loses the result without any diagnostic.
    let m = machine(
        "variables:
  - {name: result, value: \"0\"}
subStates:
  - name: A
    isInitial: true
    entryAction: Act
",
        &[("Act", "parameters:\n  - {name: reslut, direction: out}\nsubActions:\n  - {name: a, kind: AssignmentAction, assign: \"reslut := 5\"}\n")],
    );
    let r = Run::new(&m, Options::default());
    assert!(r.error().is_none());
    assert_eq!(num(&r, "result"), "0");
}

#[test]
fn a_failing_activity_stops_the_run_and_later_events_do_nothing() {
    let m = machine(
        "variables:
  - {name: d, value: \"0\"}
subStates:
  - name: A
    isInitial: true
    transitions: [{target: B, accept: Pkg::Go, effect: Boom}]
  - name: B
",
        &[("Boom", "parameters:\n  - {name: d, direction: inout}\nsubActions:\n  - {name: a, kind: AssignmentAction, assign: \"d := 1 / 0\"}\n")],
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Go", &[]);
    assert!(r.error().is_some(), "division by zero inside the activity");
    let before = r.trace().len();
    r.send("Pkg::Go", &[]);
    assert_eq!(r.trace().len(), before, "a halted run takes no more events");
}

#[test]
fn a_long_loop_inside_an_entry_activity_completes() {
    let m = machine(
        "variables:
  - {name: n, value: \"0\"}
subStates:
  - name: A
    isInitial: true
    entryAction: Spin
",
        &[("Spin", "parameters:\n  - {name: n, direction: inout}\nsubActions:\n  - name: lp\n    kind: LoopAction\n    loopKind: while\n    condition: \"n < 900\"\n    body:\n      - {name: inc, kind: AssignmentAction, assign: \"n := n + 1\"}\n")],
    );
    let r = Run::new(&m, Options::default());
    assert!(r.error().is_none(), "{:?}", r.error());
    assert_eq!(num(&r, "n"), "900");
}

#[test]
fn a_transitions_effect_activity_sees_the_variables_before_the_same_transitions_assign() {
    // Characterisation of ordering: `effect:` runs before `assign:` on one transition, so an activity named in
    // `effect:` reads the OLD value of anything the same transition assigns.
    let m = machine(
        "variables:
  - {name: u, value: \"1\"}
  - {name: seen, value: \"0\"}
subStates:
  - name: A
    isInitial: true
    transitions: [{target: B, accept: Pkg::Go, effect: Peek, assign: \"u := 50\"}]
  - name: B
",
        &[("Peek", "parameters:\n  - {name: u, direction: in}\n  - {name: seen, direction: out}\nsubActions:\n  - {name: a, kind: AssignmentAction, assign: \"seen := u\"}\n")],
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Go", &[]);
    assert_eq!((num(&r, "u"), num(&r, "seen")), ("50".into(), "1".into()));
}

// REQ-TRS-ACTSEND-001 / TC-TRS-ACTSEND-001
#[test]
fn a_send_action_inside_a_called_activity_reaches_the_machines_outbox() {
    let m = machine(
        "variables:
  - {name: v, value: \"4\"}
subStates:
  - name: A
    isInitial: true
    transitions: [{internal: true, accept: Pkg::Go, effect: Notify}]
",
        &[("Notify", "variables: []\nparameters:\n  - {name: v, direction: in}\nsubActions:\n  - {name: s, kind: SendAction, payload: Pkg::Level, parameters: [{name: value, value: \"v\"}]}\n")],
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Go", &[]);
    assert!(r.error().is_none(), "{:?}", r.error());
    let out = r.take_outbox();
    assert_eq!(out.len(), 1, "the activity's SendAction should surface as a message: outbox was {out:?}");
}

// ── systems of parts ─────────────────────────────────────────────────────────

fn sm_el(name: &str, body: &str) -> (String, String) {
    (name.to_string(), format!("---\ntype: StateDef\nname: {name}\n{body}---\n"))
}
fn act_el(name: &str, body: &str) -> (String, String) {
    (name.to_string(), format!("---\ntype: ActionDef\nname: {name}\n{body}---\n"))
}
fn def_el(name: &str, machine: &str, ports: &[(&str, &str)]) -> (String, String) {
    let ports: String = ports.iter().map(|(n, d)| format!("  - {{name: {n}, type: Port, direction: {d}}}\n")).collect();
    (name.to_string(), format!("---\ntype: PartDef\nname: {name}\nexhibitsStates: [{machine}]\nfeatures:\n{ports}---\n"))
}

fn system(files: Vec<(String, String)>, features: &str, connections: &str) -> SystemModel {
    let root = tmp("sys");
    for (n, c) in files {
        std::fs::write(root.join(format!("{n}.md")), c).unwrap();
    }
    std::fs::write(root.join("Sys.md"), format!("---\ntype: PartDef\nname: Sys\nfeatures:\n{features}connections:\n{connections}---\n")).unwrap();
    SystemModel::build_in(&walk_model(&root).unwrap(), "Sys", &root).unwrap_or_else(|e| panic!("{e:?}"))
}

fn sv(s: &System, part: &str, v: &str) -> String {
    format_value(&s.var(part, v).unwrap())
}

const CTRL_ACT: &str = "parameters:
  - {name: err, direction: in}
  - {name: u, direction: out}
subActions:
  - {name: gain, kind: AssignmentAction, assign: \"u := err * 2\"}
";

#[test]
fn a_controller_computes_its_output_in_an_activity_entered_through_a_state() {
    // The payload is copied to a variable on the transition into `Compute`; the activity runs on entry, after the
    // assign, so it sees the new value.
    let sensor = "variables:\n  - {name: reading, value: \"3\"}\nsubStates:\n  - name: S\n    isInitial: true\n    transitions:\n      - {internal: true, accept: Tick, effect: {send: Meas, via: out, payload: {v: \"reading\"}}}\n";
    let ctrl = "variables:
  - {name: err, value: \"0\"}
  - {name: u, value: \"0\"}
subStates:
  - name: Wait
    isInitial: true
    transitions:
      - {target: Compute, accept: {payload: Meas, via: inp, as: m}, assign: \"err := m.v\"}
  - name: Compute
    entryAction: Gain
    transitions:
      - {target: Wait, effect: {send: Cmd, via: out, payload: {u: \"u\"}}}
";
    let plant = "variables:\n  - {name: got, value: \"0\"}\nsubStates:\n  - name: P\n    isInitial: true\n    transitions:\n      - {internal: true, accept: {payload: Cmd, via: inp, as: c}, assign: \"got := c.u\"}\n";
    let m = system(
        vec![
            sm_el("SnM", sensor),
            sm_el("CtM", ctrl),
            sm_el("PlM", plant),
            act_el("Gain", CTRL_ACT),
            def_el("SnP", "SnM", &[("out", "out")]),
            def_el("CtP", "CtM", &[("inp", "in"), ("out", "out")]),
            def_el("PlP", "PlM", &[("inp", "in")]),
        ],
        "  - {name: sn, typedBy: SnP}\n  - {name: ct, typedBy: CtP}\n  - {name: pl, typedBy: PlP}\n",
        "  - {from: sn.out, to: ct.inp}\n  - {from: ct.out, to: pl.inp}\n",
    );
    let mut s = System::new(&m, Options::default());
    assert!(s.error().is_none(), "{:?}", s.error());
    s.receive("sn", None, "Tick", &[]);
    assert!(s.error().is_none(), "{:?}", s.error());
    assert_eq!(sv(&s, "ct", "u"), "6");
    assert_eq!(sv(&s, "pl", "got"), "6", "sensor → controller (activity) → plant, all in one instant");
    assert_eq!(s.active("ct").unwrap(), ["Wait"]);
}

#[test]
fn a_hundred_round_trips_with_connection_delays_land_at_the_right_instants() {
    let ping = "variables:\n  - {name: n, value: \"0\"}\ninitialEffect: {send: Ping, via: out, payload: {n: \"0\"}}\nsubStates:\n  - name: S\n    isInitial: true\n    transitions:\n      - {internal: true, accept: {payload: Pong, via: inp, as: m}, guard: \"m.n < 100\", assign: \"n := m.n\", effect: {send: Ping, via: out, payload: {n: \"m.n\"}}}\n";
    let pong = "variables:\n  - {name: seen, value: \"0\"}\nsubStates:\n  - name: S\n    isInitial: true\n    transitions:\n      - {internal: true, accept: {payload: Ping, via: inp, as: m}, assign: \"seen := m.n + 1\", effect: {send: Pong, via: out, payload: {n: \"m.n + 1\"}}}\n";
    let m = system(
        vec![
            sm_el("PgM", ping),
            sm_el("PnM", pong),
            def_el("PgP", "PgM", &[("out", "out"), ("inp", "in")]),
            def_el("PnP", "PnM", &[("out", "out"), ("inp", "in")]),
        ],
        "  - {name: a, typedBy: PgP}\n  - {name: b, typedBy: PnP}\n",
        "  - {from: a.out, to: b.inp, delay: \"1 ms\"}\n  - {from: b.out, to: a.inp, delay: \"1 ms\"}\n",
    );
    let mut s = System::new(&m, Options::default());
    s.advance(1.0);
    assert!(s.error().is_none(), "{:?}", s.error());
    assert_eq!(sv(&s, "b", "seen"), "100", "pong saw the 100th ping");
    assert_eq!(sv(&s, "a", "n"), "99");
    let last = s.messages().iter().filter_map(|m| m.delivered).fold(0.0_f64, f64::max);
    assert!((last - 0.200).abs() < 1e-9, "100 pings + 100 pongs at 1 ms each: last delivery at 200 ms, got {last}");
}

#[test]
fn three_parts_in_a_ring_pass_a_token_with_distinct_delays_without_losing_order() {
    let node = |me: i32| {
        format!("variables:\n  - {{name: hops, value: \"0\"}}\nsubStates:\n  - name: S\n    isInitial: true\n    transitions:\n      - {{internal: true, accept: {{payload: Tok, via: inp, as: t}}, guard: \"t.n < 9\", assign: \"hops := hops + 1\", effect: {{send: Tok, via: out, payload: {{n: \"t.n + 1\"}}}}}}\n# {me}\n")
    };
    let starter = "variables:\n  - {name: hops, value: \"0\"}\ninitialEffect: {send: Tok, via: out, payload: {n: \"0\"}}\nsubStates:\n  - name: S\n    isInitial: true\n    transitions:\n      - {internal: true, accept: {payload: Tok, via: inp, as: t}, guard: \"t.n < 9\", assign: \"hops := hops + 1\", effect: {send: Tok, via: out, payload: {n: \"t.n + 1\"}}}\n";
    let m = system(
        vec![
            sm_el("N0M", starter),
            sm_el("N1M", &node(1)),
            sm_el("N2M", &node(2)),
            def_el("N0P", "N0M", &[("out", "out"), ("inp", "in")]),
            def_el("N1P", "N1M", &[("out", "out"), ("inp", "in")]),
            def_el("N2P", "N2M", &[("out", "out"), ("inp", "in")]),
        ],
        "  - {name: n0, typedBy: N0P}\n  - {name: n1, typedBy: N1P}\n  - {name: n2, typedBy: N2P}\n",
        "  - {from: n0.out, to: n1.inp, delay: \"3 ms\"}\n  - {from: n1.out, to: n2.inp, delay: \"2 ms\"}\n  - {from: n2.out, to: n0.inp, delay: \"1 ms\"}\n",
    );
    let mut s = System::new(&m, Options::default());
    s.advance(1.0);
    assert!(s.error().is_none(), "{:?}", s.error());
    let hops: i32 = ["n0", "n1", "n2"].iter().map(|p| sv(&s, p, "hops").parse::<i32>().unwrap()).sum();
    assert_eq!(hops, 9, "token n=0..8 is forwarded 9 times, the 9th hop (n=9) stops");
    let times: Vec<f64> = s.messages().iter().filter_map(|m| m.delivered).collect();
    assert!(times.windows(2).all(|w| w[0] <= w[1]), "deliveries are in time order: {times:?}");
}

#[test]
fn a_fan_in_of_simultaneous_messages_is_delivered_in_connection_order() {
    let tx = "subStates:\n  - name: S\n    isInitial: true\n    transitions:\n      - {internal: true, accept: Tick, effect: {send: M, via: out, payload: {id: \"1\"}}}\n";
    let rx = "variables:\n  - {name: log, value: \"0\"}\nsubStates:\n  - name: S\n    isInitial: true\n    transitions:\n      - {internal: true, accept: {payload: M, via: inp, as: m}, assign: \"log := log * 10 + m.id\"}\n";
    let tx2 = tx.replace("\"1\"", "\"2\"");
    let m = system(
        vec![
            sm_el("T1M", tx),
            sm_el("T2M", &tx2),
            sm_el("RxM", rx),
            def_el("T1P", "T1M", &[("out", "out")]),
            def_el("T2P", "T2M", &[("out", "out")]),
            def_el("RxP", "RxM", &[("inp", "in")]),
        ],
        "  - {name: t1, typedBy: T1P}\n  - {name: t2, typedBy: T2P}\n  - {name: rx, typedBy: RxP}\n",
        "  - {from: t2.out, to: rx.inp}\n  - {from: t1.out, to: rx.inp}\n",
    );
    let mut s = System::new(&m, Options::default());
    s.receive("t1", None, "Tick", &[]);
    s.receive("t2", None, "Tick", &[]);
    assert!(s.error().is_none(), "{:?}", s.error());
    assert_eq!(sv(&s, "rx", "log"), "12", "t1 ticked first, so its message is delivered first");
    let _ = parse_value("1");
}

// ── REQ-TRS-ACTSEND-001: sends made by activities ───────────────────────────

#[test]
fn an_effect_activitys_send_carries_event_port_and_typed_payload_and_shows_in_the_trace() {
    let m = machine(
        "variables:\n  - {name: v, value: \"4\", unit: \"SI::m\"}\nsubStates:\n  - name: A\n    isInitial: true\n    transitions: [{internal: true, accept: Pkg::Go, effect: Notify}]\n",
        &[("Notify", "variables: []\nparameters:\n  - {name: v, direction: in, unit: \"SI::m\"}\nsubActions:\n  - {name: s, kind: SendAction, payload: Pkg::Level, via: p, parameters: [{name: value, value: \"v * 2\"}]}\n")],
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Go", &[]);
    assert!(r.error().is_none(), "{:?}", r.error());
    let out = r.take_outbox();
    assert_eq!(out.len(), 1);
    assert_eq!((out[0].event.as_str(), out[0].via.as_deref()), ("Pkg::Level", Some("p")));
    assert_eq!(format_value(&out[0].payload[0].1), format_value(&parse_value("8 m").unwrap()), "typed, unit kept");
    let step = r.trace().iter().rev().find(|s| s.transition.is_some()).unwrap();
    assert!(step.actions.iter().any(|a| a.starts_with("send Pkg::Level via p (value=")), "{:?}", step.actions);
}

#[test]
fn sends_come_out_in_firing_order_around_the_machines_own_send() {
    let m = machine(
        "subStates:
  - name: A
    isInitial: true
    transitions: [{target: B, accept: Pkg::Go, effect: {send: Mid}}]
  - name: B
    entryAction: Two
",
        &[("Two", "variables: []\nsubActions:\n  - {name: a, kind: SendAction, payload: Pkg::X}\n  - {name: b, kind: SendAction, payload: Pkg::Y}\nsuccessionConnections:\n  - {after: a, before: b}\n")],
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Go", &[]);
    let names: Vec<String> = r.take_outbox().into_iter().map(|s| s.event).collect();
    assert_eq!(names, ["Mid", "Pkg::X", "Pkg::Y"], "effect, then the entry activity's two sends");
    let step = r.trace().iter().rev().find(|s| s.transition.is_some()).unwrap();
    let sends: Vec<&String> = step.actions.iter().filter(|a| a.starts_with("send ")).collect();
    assert_eq!(sends.len(), 3, "{:?}", step.actions);
}

#[test]
fn sends_in_a_loop_a_performed_callee_and_a_taken_branch_are_emitted_and_an_untaken_one_is_not() {
    let m = machine(
        "variables:\n  - {name: i, value: \"0\"}\n  - {name: go, value: \"1\"}\nsubStates:\n  - name: A\n    isInitial: true\n    transitions: [{internal: true, accept: Pkg::Go, effect: Outer}]\n",
        &[
            ("Inner", "subActions:\n  - {name: s, kind: SendAction, payload: Pkg::FromCallee}\n"),
            (
                "Outer",
                "parameters:
  - {name: i, direction: inout}
  - {name: go, direction: in}
subActions:
  - name: lp
    kind: LoopAction
    loopKind: while
    condition: \"i < 2\"
    body:
      - {name: inc, kind: AssignmentAction, assign: \"i := i + 1\"}
      - {name: tick, kind: SendAction, payload: Pkg::Tick, parameters: [{name: n, value: i}]}
  - {name: call, kind: PerformAction, typedBy: Inner}
  - name: br
    kind: IfAction
    condition: \"go == 1\"
    then:
      - {name: yes, kind: SendAction, payload: Pkg::Yes}
    else:
      - {name: no, kind: SendAction, payload: Pkg::No}
successionConnections:
  - {after: lp, before: call}
  - {after: call, before: br}
",
            ),
        ],
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Go", &[]);
    assert!(r.error().is_none(), "{:?}", r.error());
    let names: Vec<String> = r.take_outbox().into_iter().map(|s| s.event).collect();
    assert_eq!(names, ["Pkg::Tick", "Pkg::Tick", "Pkg::FromCallee", "Pkg::Yes"]);
}

#[test]
fn a_failing_or_waiting_activity_emits_none_of_its_sends() {
    for body in [
        "variables:\n  - {name: z, value: \"0\"}\nsubActions:\n  - {name: s, kind: SendAction, payload: Pkg::Early}\n  - {name: boom, kind: AssignmentAction, assign: \"z := 1 / 0\"}\nsuccessionConnections:\n  - {after: s, before: boom}\n",
        "variables: []\nsubActions:\n  - {name: s, kind: SendAction, payload: Pkg::Early}\n  - {name: w, kind: AcceptAction, payload: Pkg::Never}\nsuccessionConnections:\n  - {after: s, before: w}\n",
    ] {
        let m = machine(
            "subStates:\n  - name: A\n    isInitial: true\n    transitions: [{internal: true, accept: Pkg::Go, effect: Bad}]\n",
            &[("Bad", body)],
        );
        let mut r = Run::new(&m, Options::default());
        r.send("Pkg::Go", &[]);
        assert!(r.error().is_some(), "no error for body: {body}");
        assert!(r.take_outbox().is_empty(), "nothing from the failed activity");
    }
}

#[test]
fn a_part_whose_behaviour_is_an_activity_talks_to_its_neighbour_through_the_connection_with_its_delay() {
    let tx = "subStates:\n  - name: S\n    isInitial: true\n    transitions: [{internal: true, accept: Tick, effect: Say}]\n";
    let rx = "variables:\n  - {name: got, value: \"0\"}\nsubStates:\n  - name: S\n    isInitial: true\n    transitions:\n      - {internal: true, accept: {payload: Hello, via: inp, as: m}, assign: \"got := m.n\"}\n";
    let say = "variables: []\nsubActions:\n  - {name: s, kind: SendAction, payload: Hello, via: out, parameters: [{name: n, value: \"41 + 1\"}]}\n";
    let m = system(
        vec![
            sm_el("TxM", tx),
            sm_el("RxM", rx),
            act_el("Say", say),
            def_el("TxP", "TxM", &[("out", "out")]),
            def_el("RxP", "RxM", &[("inp", "in")]),
        ],
        "  - {name: tx, typedBy: TxP}\n  - {name: rx, typedBy: RxP}\n",
        "  - {from: tx.out, to: rx.inp, delay: \"5 ms\"}\n",
    );
    let mut s = System::new(&m, Options::default());
    s.receive("tx", None, "Tick", &[]);
    assert!(s.error().is_none(), "{:?}", s.error());
    assert_eq!(sv(&s, "rx", "got"), "0", "not yet: 5 ms delay");
    s.advance(0.006);
    assert_eq!(sv(&s, "rx", "got"), "42");
}


#[test]
fn an_activity_that_only_sends_is_executable_without_declaring_anything() {
    // GH #516: a SendAction makes an ActionDef executable; no `variables:` opt-in
    let m = machine(
        "subStates:\n  - name: A\n    isInitial: true\n    transitions: [{internal: true, accept: Pkg::Go, effect: Draw}]\n",
        &[("Draw", "subActions:\n  - {name: s, kind: SendAction, payload: Pkg::X}\n")],
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Go", &[]);
    assert!(r.error().is_none(), "{:?}", r.error());
    let out = r.take_outbox();
    assert_eq!(out.len(), 1, "{out:?}");
    assert_eq!(out[0].event, "Pkg::X");
}

#[test]
fn an_activity_with_no_send_variables_or_assignment_stays_a_recorded_reference() {
    let m = machine(
        "subStates:\n  - name: A\n    isInitial: true\n    transitions: [{internal: true, accept: Pkg::Go, effect: Draw}]\n",
        &[("Draw", "subActions:\n  - {name: s, kind: Action}\n")],
    );
    let mut r = Run::new(&m, Options::default());
    r.send("Pkg::Go", &[]);
    assert!(r.error().is_none(), "{:?}", r.error());
    assert!(r.take_outbox().is_empty());
    let step = r.trace().iter().rev().find(|s| s.transition.is_some()).unwrap();
    assert!(step.actions.iter().any(|a| a == "behavior Draw"), "{:?}", step.actions);
}
