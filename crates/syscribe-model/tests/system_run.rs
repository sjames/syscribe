//! TC-TRS-SYSCOMP-001 / REQ-TRS-SYSCOMP-001 (GH #297): executing a composite — messages, delays, bindings, ordering.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::expr::{format_value, parse_value};
use syscribe_model::sm_run::Options;
use syscribe_model::system_run::{System, SystemModel};
use syscribe_model::walker::walk_model;

fn sm(name: &str, body: &str) -> (String, String) {
    (name.to_string(), format!("---\ntype: StateDef\nname: {name}\n{body}---\n"))
}

fn def(name: &str, machine: &str, ports: &[(&str, &str)]) -> (String, String) {
    let ports: String = ports.iter().map(|(n, d)| format!("  - {{name: {n}, type: Port, direction: {d}}}\n")).collect();
    (name.to_string(), format!("---\ntype: PartDef\nname: {name}\nexhibitsStates: [{machine}]\nfeatures:\n{ports}---\n"))
}

fn system(files: Vec<(String, String)>, features: &str, connections: &str, bindings: &str) -> SystemModel {
    static N: AtomicU64 = AtomicU64::new(0);
    let root: PathBuf = std::env::temp_dir().join(format!("syscribe-sysrun-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    for (n, c) in files {
        std::fs::write(root.join(format!("{n}.md")), c).unwrap();
    }
    std::fs::write(root.join("Sys.md"), format!("---\ntype: PartDef\nname: Sys\nfeatures:\n{features}connections:\n{connections}bindingConnections:\n{bindings}---\n")).unwrap();
    SystemModel::build_in(&walk_model(&root).unwrap(), "Sys", &root).unwrap_or_else(|e| panic!("{e:?}"))
}

const TX: &str = "variables:
  - {name: level, value: \"7\"}
  - {name: dist, value: \"0\", unit: \"SI::m\"}
subStates:
  - name: Idle
    isInitial: true
    transitions:
      - {internal: true, accept: Tick, effect: {send: Cmd, via: out, payload: {n: \"level\"}}}
";
const RX: &str = "variables:
  - {name: got, value: \"0\"}
  - {name: dist, value: \"0\", unit: \"SI::m\"}
subStates:
  - name: Waiting
    isInitial: true
    transitions:
      - {target: Done, accept: {payload: Cmd, via: inp, as: m}, assign: \"got := m.n\"}
  - name: Done
";

fn pair(connections: &str, bindings: &str) -> SystemModel {
    system(
        vec![sm("TxM", TX), sm("RxM", RX), def("TxP", "TxM", &[("out", "out")]), def("RxP", "RxM", &[("inp", "in")])],
        "  - {name: tx, typedBy: TxP}\n  - {name: rx, typedBy: RxP}\n",
        connections,
        bindings,
    )
}

fn n(s: &System, part: &str, v: &str) -> String {
    format_value(&s.var(part, v).unwrap())
}

#[test]
fn a_message_is_delivered_with_its_payload_and_the_receiver_reacts() {
    let m = pair("  - {from: tx.out, to: rx.inp}\n", "");
    let mut s = System::new(&m, Options::default());
    assert_eq!(s.active("rx").unwrap(), ["Waiting"]);
    s.receive("tx", None, "Tick", &[]);
    assert_eq!(s.active("rx").unwrap(), ["Done"]);
    assert_eq!(n(&s, "rx", "got"), format_value(&parse_value("7").unwrap()));
    let msgs = s.messages();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].delivered, Some(0.0));
    assert!(s.describe(&msgs[0]).contains("delivered to rx.inp"));
}

#[test]
fn a_connection_delay_postpones_delivery_to_exactly_that_instant() {
    let m = pair("  - {from: tx.out, to: rx.inp, delay: \"5 ms\"}\n", "");
    let mut s = System::new(&m, Options::default());
    s.receive("tx", None, "Tick", &[]);
    s.advance(0.004);
    assert_eq!(s.active("rx").unwrap(), ["Waiting"], "not yet at 4 ms");
    s.advance(0.002);
    assert_eq!(s.active("rx").unwrap(), ["Done"]);
    let d = s.messages()[0].delivered.unwrap();
    assert!((d - 0.005).abs() < 1e-12, "delivered at {d}");
}

#[test]
fn one_output_fans_out_to_every_connected_input_in_connection_order() {
    let m = system(
        vec![sm("TxM", TX), sm("RxM", RX), def("TxP", "TxM", &[("out", "out")]), def("RxP", "RxM", &[("inp", "in")])],
        "  - {name: tx, typedBy: TxP}\n  - {name: a, typedBy: RxP}\n  - {name: b, typedBy: RxP}\n",
        "  - {from: tx.out, to: b.inp}\n  - {from: tx.out, to: a.inp}\n",
        "",
    );
    let mut s = System::new(&m, Options::default());
    s.receive("tx", None, "Tick", &[]);
    assert_eq!((s.active("a").unwrap(), s.active("b").unwrap()), (vec!["Done".to_string()], vec!["Done".to_string()]));
    let order: Vec<&str> = s.messages().iter().map(|m| ["tx", "a", "b"][m.to.as_ref().unwrap().0]).collect();
    assert_eq!(order, ["b", "a"], "connection order, not part order");
}

#[test]
fn bound_values_follow_their_source_and_reach_the_sink_machine() {
    let m = pair("  - {from: tx.out, to: rx.inp}\n", "  - {from: tx.dist, to: rx.dist}\n");
    let mut s = System::new(&m, Options::default());
    s.set("tx", "dist", parse_value("2.5 m").unwrap());
    assert_eq!(n(&s, "rx", "dist"), format_value(&parse_value("2.5 m").unwrap()));
}

#[test]
fn timers_of_a_part_drive_periodic_messages_on_the_shared_clock() {
    let ticker = "subStates:
  - name: Run
    isInitial: true
    transitions:
      - {internal: true, accept: {after: \"1 s\"}, effect: {send: Beat, via: out}}
";
    let m = system(vec![sm("TkM", ticker), def("TkP", "TkM", &[("out", "out")])], "  - {name: tk, typedBy: TkP}\n", "", "");
    let mut s = System::new(&m, Options::default());
    s.advance(3.5);
    // an internal transition does not re-arm its timer in this machine, so only the first beat is sent
    let times: Vec<f64> = s.messages().iter().map(|m| m.sent).collect();
    assert!(!times.is_empty() && (times[0] - 1.0).abs() < 1e-9, "{times:?}");
    assert_eq!(s.messages()[0].to, None, "nothing is connected to the port");
    assert!(s.describe(&s.messages()[0]).contains("connected to nothing"));
}

#[test]
fn parts_that_keep_answering_each_other_end_in_a_livelock_error_not_a_hang() {
    let ping = "initialEffect: {send: Ping, via: out}
subStates:
  - name: S
    isInitial: true
    transitions:
      - {internal: true, accept: {payload: Pong, via: inp}, effect: {send: Ping, via: out}}
";
    let pong = "subStates:
  - name: S
    isInitial: true
    transitions:
      - {internal: true, accept: {payload: Ping, via: inp}, effect: {send: Pong, via: out}}
";
    let m = system(
        vec![sm("PingM", ping), sm("PongM", pong), def("PingP", "PingM", &[("out", "out"), ("inp", "in")]), def("PongP", "PongM", &[("out", "out"), ("inp", "in")])],
        "  - {name: a, typedBy: PingP}\n  - {name: b, typedBy: PongP}\n",
        "  - {from: a.out, to: b.inp}\n  - {from: b.out, to: a.inp}\n",
        "",
    );
    let s = System::new(&m, Options::default());
    assert!(s.error().is_some_and(|e| e.contains("livelock")), "{:?}", s.error());
}

#[test]
fn the_hash_covers_machines_and_wiring_and_is_stable() {
    let a = pair("  - {from: tx.out, to: rx.inp}\n", "");
    let b = pair("  - {from: tx.out, to: rx.inp}\n", "");
    let c = pair("  - {from: tx.out, to: rx.inp, delay: \"1 ms\"}\n", "");
    assert_eq!(a.hash, b.hash);
    assert_ne!(a.hash, c.hash);
}

#[test]
fn a_timer_is_not_lost_to_float_drift_when_another_part_makes_the_clock_stop_early() {
    let slow = "subStates:\n  - name: Wait\n    isInitial: true\n    transitions:\n      - {target: Timeout, accept: {after: \"14 ms\"}}\n  - {name: Timeout, isFinal: true}\n";
    let fast = "subStates:\n  - name: Wait\n    isInitial: true\n    transitions:\n      - {target: Hit, accept: {after: \"3 ms\"}}\n  - {name: Hit, isFinal: true}\n";
    let m = system(vec![sm("SlowM", slow), sm("FastM", fast), def("SlowP", "SlowM", &[]), def("FastP", "FastM", &[])], "  - {name: a, typedBy: SlowP}\n  - {name: b, typedBy: FastP}\n", "", "");
    let mut s = System::new(&m, Options::default());
    s.advance(0.014);
    assert_eq!(s.active("b").unwrap(), ["Hit"]);
    assert_eq!(s.active("a").unwrap(), ["Timeout"], "0.003 + (0.014 - 0.003) is one ulp short of 0.014");
}

#[test]
fn a_halted_sink_does_not_make_the_bindings_look_unconverged() {
    let src = "variables:\n  - {name: x, value: \"0\"}\nsubStates:\n  - {name: A, isInitial: true}\n";
    let sink = "variables:\n  - {name: x, value: \"0\"}\nsubStates:\n  - name: A\n    isInitial: true\n    transitions:\n      - {target: Z, accept: Stop}\n  - {name: Z, isFinal: true}\n";
    let m = system(vec![sm("SrcM", src), sm("SinkM", sink), def("SrcP", "SrcM", &[]), def("SinkP", "SinkM", &[])], "  - {name: s, typedBy: SrcP}\n  - {name: k, typedBy: SinkP}\n", "", "  - {from: s.x, to: k.x}\n");
    let mut s = System::new(&m, Options::default());
    s.receive("k", None, "Stop", &[]);
    s.set("s", "x", parse_value("5").unwrap());
    assert!(s.error().is_none(), "{:?}", s.error());
}

#[test]
fn inferring_a_subject_from_satisfies_still_picks_the_single_state_machine() {
    use syscribe_model::sm_scen::subject_of;
    let root = std::env::temp_dir().join(format!("syscribe-subj-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(root.join("REQ-A-001.md"), "---\ntype: Requirement\nid: REQ-A-001\nname: a\nstatus: approved\nreqDomain: software\n---\n\nShall.\n").unwrap();
    std::fs::write(root.join("M.md"), "---\ntype: StateDef\nname: M\nsatisfies: [REQ-A-001]\nsubStates:\n  - {name: A, isInitial: true}\n---\n").unwrap();
    std::fs::write(root.join("P.md"), "---\ntype: PartDef\nname: P\nsatisfies: [REQ-A-001]\n---\nPart.\n").unwrap();
    std::fs::write(root.join("TC-A-001.md"), "---\ntype: TestCase\nid: TC-A-001\nname: t\nstatus: active\ntestLevel: L2\nverifies: [REQ-A-001]\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then 1 == 1\n```\n").unwrap();
    let elems = walk_model(&root).unwrap();
    let tc = elems.iter().find(|e| e.frontmatter.id.as_deref() == Some("TC-A-001")).unwrap();
    assert_eq!(subject_of(&elems, tc).unwrap().qualified_name, "M");
}
