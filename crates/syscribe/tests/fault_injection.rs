//! TC-TRS-FAULT-001 / REQ-TRS-FAULT-001: failure modes of FMEA entries and fault-tree events injected into a composite.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const FMEA: &str = "---
type: FMEASheet
id: FMEA-LINK-001
name: \"FMEA of the link\"
status: draft
entries:
  - {id: FM-TX-001, ref: TxP, failureMode: \"command lost\", injection: [{kind: drop, port: out}]}
  - {id: FM-TX-002, ref: TxP, failureMode: \"command late\", injection: [{kind: delay, port: out, event: Cmd, by: \"10 ms\"}]}
  - {id: FM-TX-003, ref: TxP, failureMode: \"level stuck\", injection: [{kind: stuck, variable: level, value: \"3\"}]}
  - {id: FM-TX-004, ref: TxP, failureMode: \"level biased\", injection: [{kind: offset, variable: level, by: \"2\"}]}
  - {id: FM-TX-005, ref: TxP, failureMode: \"payload corrupted\", injection: [{kind: corrupt, port: out, event: Cmd, field: n, value: \"99\"}]}
  - {id: FM-RX-001, ref: RxP, failureMode: \"receiver dead\", injection: [{kind: silent}]}
  - {id: FM-RX-002, ref: RxP, failureMode: \"an analysis row without a fault model\"}
---
";

const FILES: &[(&str, &str)] = &[
    ("TxM", "---\ntype: StateDef\nname: TxM\nvariables:\n  - {name: level, value: \"7\"}\nsubStates:\n  - name: Idle\n    isInitial: true\n    transitions:\n      - {target: Idle, accept: Tick, effect: {send: Cmd, via: out, payload: {n: \"level\"}}}\n---\n"),
    ("RxM", "---\ntype: StateDef\nname: RxM\nvariables:\n  - {name: got, value: \"0\"}\nsubStates:\n  - name: Waiting\n    isInitial: true\n    transitions:\n      - {target: Done, accept: {payload: Cmd, via: inp, as: m}, assign: \"got := m.n\"}\n  - {name: Done, isFinal: true}\n---\n"),
    ("TxP", "---\ntype: PartDef\nname: TxP\nexhibitsStates: [TxM]\nfeatures:\n  - {name: out, type: Port, direction: out}\n---\nTransmitter.\n"),
    ("RxP", "---\ntype: PartDef\nname: RxP\nexhibitsStates: [RxM]\nfeatures:\n  - {name: inp, type: Port, direction: in}\n---\nReceiver.\n"),
    ("Sys", "---\ntype: PartDef\nname: Sys\nfeatures:\n  - {name: tx, typedBy: TxP}\n  - {name: rx, typedBy: RxP}\nconnections:\n  - {from: tx.out, to: rx.inp}\n---\nThe system.\n"),
    ("REQ-SYS-001", "---\ntype: Requirement\nid: REQ-SYS-001\nname: delivery\nstatus: approved\nreqDomain: software\n---\n\nThe receiver shall get the command.\n"),
    ("FMEA-LINK-001", FMEA),
];

const TC: &str = "---
type: TestCase
id: TC-SYS-001
name: system under faults
status: active
testLevel: L3
subject: Sys
verifies: [REQ-SYS-001]
---

```gherkin
Feature: faults

  Scenario: a lost command
    Given fault FM-TX-001 is injected
    When tx receives Tick
    Then tx sent Cmd via out
    And rx is in state Waiting

  Scenario: a cleared fault
    Given fault FM-TX-001 is injected
    When fault FM-TX-001 is cleared
    And tx receives Tick
    Then rx is in state Done

  Scenario: a late command
    Given fault FM-TX-002 is injected
    When tx receives Tick
    And 5 ms elapses
    Then rx is in state Waiting
    When 6 ms elapses
    Then rx is in state Done

  Scenario: a stuck sensor
    Given fault FM-TX-003 is injected
    And tx.level is 9
    When tx receives Tick
    Then tx.level == 3
    And rx.got == 3

  Scenario: a biased sensor
    Given tx.level is 5
    When fault FM-TX-004 is injected
    Then tx.level == 7

  Scenario: a corrupted payload
    Given fault FM-TX-005 is injected
    When tx receives Tick
    Then rx.got == 99

  Scenario: a dead receiver
    Given fault FM-RX-001 is injected
    When tx receives Tick
    Then rx is in state Waiting
```
";

fn model(tc: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-fault-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    for (n, c) in FILES {
        std::fs::write(root.join(format!("{n}.md")), c).unwrap();
    }
    std::fs::write(root.join("TC-SYS-001.md"), tc).unwrap();
    root
}

fn run(root: &PathBuf, args: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (o.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

#[test]
fn every_fault_kind_behaves_as_specified() {
    let (c, t) = run(&model(TC), &["simulate", "TC-SYS-001", "--json"]);
    let j: serde_json::Value = serde_json::from_str(&t).unwrap_or_else(|e| panic!("{e}: {t}"));
    let v = j["verdicts"].as_array().unwrap();
    let failed: Vec<String> = v.iter().filter(|x| x["status"] != "pass").map(|x| format!("{}: {}", x["scenario"], x["message"])).collect();
    assert!(failed.is_empty() && c == 0, "{failed:?}\n{t}");
}

#[test]
fn injections_and_clearings_are_recorded_in_the_verdict() {
    let (_, t) = run(&model(TC), &["simulate", "TC-SYS-001", "--json", "--scenario", "a cleared fault"]);
    let j: serde_json::Value = serde_json::from_str(&t).unwrap();
    let msgs: Vec<String> = j["verdicts"][0]["messages"].as_array().unwrap().iter().map(|m| m.as_str().unwrap().to_string()).collect();
    assert!(msgs.iter().any(|m| m.contains("fault FM-TX-001 injected at 0 s")), "{msgs:?}");
    assert!(msgs.iter().any(|m| m.contains("fault FM-TX-001 cleared at 0 s")), "{msgs:?}");
}

#[test]
fn a_silent_part_cannot_be_cleared_and_unknown_faults_are_errors() {
    let tc = TC.replace("Scenario: a dead receiver\n    Given fault FM-RX-001 is injected\n    When tx receives Tick\n    Then rx is in state Waiting", "Scenario: a dead receiver\n    Given fault FM-RX-001 is injected\n    When fault FM-RX-001 is cleared");
    let (_, t) = run(&model(&tc), &["simulate", "TC-SYS-001", "--scenario", "a dead receiver"]);
    assert!(t.contains("cannot be cleared"), "{t}");
    let tc = TC.replace("FM-TX-001 is injected\n    When tx receives Tick\n    Then tx sent", "FM-NOPE-001 is injected\n    When tx receives Tick\n    Then tx sent");
    let (c, t) = run(&model(&tc), &["validate"]);
    assert_eq!(c, 1, "{t}");
    assert!(t.contains("E937") && t.contains("FM-NOPE-001"), "{t}");
    let (c, t) = run(&model(&tc), &["simulate", "TC-SYS-001", "--scenario", "a lost command"]);
    assert_ne!(c, 0, "{t}");
}

#[test]
fn malformed_injections_are_e936_and_prose_rows_are_untouched() {
    let (_, clean) = run(&model(TC), &["validate"]);
    assert!(!clean.contains("E936"), "{clean}");
    let root = model(TC);
    let bad = FMEA
        .replace("kind: drop, port: out", "kind: drop")
        .replace("kind: stuck, variable: level, value: \"3\"", "kind: levitate")
        .replace("by: \"10 ms\"", "by: \"soon\"");
    std::fs::write(root.join("FMEA-LINK-001.md"), bad).unwrap();
    let (c, t) = run(&root, &["validate"]);
    assert_eq!(c, 1, "{t}");
    assert!(t.matches("E936").count() >= 3, "a drop without a port, an unknown kind, a delay that is not a duration: {t}");
}

#[test]
fn a_fault_tree_event_carries_a_fault_model_too() {
    let root = model(TC);
    std::fs::create_dir_all(root.join("FT-1")).unwrap();
    std::fs::write(root.join("FT-1.md"), "---\ntype: FaultTree\nid: FT-1\nname: tree\nstatus: draft\n---\nTree.\n").unwrap();
    std::fs::write(root.join("FT-1/FTE-TX-001.md"), "---\ntype: FaultTreeEvent\nid: FTE-TX-001\nname: \"transmitter dead\"\neventKind: basic\nref: TxP\nfailureRate: 1.0e-7\ninjection:\n  - {kind: silent}\n---\nThe transmitter fails silent.\n").unwrap();
    let tc = TC.replace("Scenario: a dead receiver", "Scenario: a dead transmitter\n    Given fault FTE-TX-001 is injected\n    When tx receives Tick\n    Then rx is in state Waiting\n\n  Scenario: a dead receiver");
    std::fs::write(root.join("TC-SYS-001.md"), tc).unwrap();
    let (c, t) = run(&root, &["simulate", "TC-SYS-001", "--scenario", "a dead transmitter"]);
    assert_eq!(c, 0, "{t}");
    let (_, v) = run(&root, &["validate"]);
    assert!(!v.contains("E936") && !v.contains("W047"), "the field is known on fault-tree events: {v}");
}

#[test]
fn a_fault_that_does_not_fit_the_composite_is_an_error_and_one_for_another_part_is_inert() {
    // a stuck value of the wrong kind for the variable it holds
    let root = model(TC);
    let sheet = FMEA.replace("value: \"3\"", "value: \"true\"");
    std::fs::write(root.join("FMEA-LINK-001.md"), sheet).unwrap();
    let (c, t) = run(&root, &["simulate", "TC-SYS-001"]);
    assert_ne!(c, 0, "{t}");
    assert!(t.contains("E936") && t.contains("FM-TX-003"), "{t}");
    // a row about a part that is not in the composite never applies to it
    let root = model(TC);
    let other = FMEA.replace("ref: RxP, failureMode: \"receiver dead\"", "ref: NobodyP, failureMode: \"receiver dead\"");
    std::fs::write(root.join("FMEA-LINK-001.md"), other).unwrap();
    let (_, t) = run(&root, &["simulate", "TC-SYS-001", "--scenario", "a dead receiver"]);
    assert!(t.contains("no fault 'FM-RX-001' applies"), "{t}");
}

#[test]
fn clearing_one_fault_keeps_the_other_fault_on_the_same_variable() {
    let tc = TC.replace("  Scenario: a lost command", "  Scenario: overlapping value faults\n    Given fault FM-TX-003 is injected\n    And fault FM-TX-004 is injected\n    When fault FM-TX-003 is cleared\n    And tx.level becomes 10\n    Then tx.level == 12\n\n  Scenario: a lost command");
    let (_, t) = run(&model(&tc), &["simulate", "TC-SYS-001", "--json", "--scenario", "overlapping value faults"]);
    let j: serde_json::Value = serde_json::from_str(&t).unwrap_or_else(|e| panic!("{e}: {t}"));
    assert_eq!(j["verdicts"][0]["status"], "pass", "the offset of FM-TX-004 still applies after FM-TX-003 is cleared: {t}");
}

#[test]
fn an_unfitting_fault_fails_only_the_scenarios_that_inject_it() {
    let root = model(TC);
    std::fs::write(root.join("FMEA-LINK-001.md"), FMEA.replace("value: \"3\"", "value: \"true\"")).unwrap();
    let (_, t) = run(&root, &["simulate", "TC-SYS-001", "--json"]);
    let j: serde_json::Value = serde_json::from_str(&t).unwrap();
    let by = |title: &str| j["verdicts"].as_array().unwrap().iter().find(|v| v["scenario"] == title).unwrap().clone();
    assert_eq!(by("a lost command")["status"], "pass", "unrelated scenarios still run");
    assert_eq!(by("a stuck sensor")["status"], "error");
    assert!(by("a stuck sensor")["message"].as_str().unwrap().contains("E936"), "{t}");
}

#[test]
fn the_fault_model_is_part_of_the_evidence_hash() {
    let a = run(&model(TC), &["simulate", "TC-SYS-001", "--json", "--scenario", "a lost command"]).1;
    let root = model(TC);
    std::fs::write(root.join("FMEA-LINK-001.md"), FMEA.replace("by: \"10 ms\"", "by: \"20 ms\"")).unwrap();
    let b = run(&root, &["simulate", "TC-SYS-001", "--json", "--scenario", "a lost command"]).1;
    let h = |t: &str| serde_json::from_str::<serde_json::Value>(t).unwrap()["machineHash"].as_str().unwrap().to_string();
    assert_ne!(h(&a), h(&b), "changing a fault changes what a run can show, so it changes the hash");
}

#[test]
fn a_part_may_be_called_fault() {
    use syscribe_model::sm_scen::Kw;
    use syscribe_model::system_scen::{parse_step, SCmd};
    assert!(matches!(parse_step(Kw::Then, "fault is in state Idle"), Ok(SCmd::ExpectState(..))));
    assert!(matches!(parse_step(Kw::When, "fault receives Tick"), Ok(SCmd::Receive { .. })));
    assert!(matches!(parse_step(Kw::When, "fault FM-1 is cleared"), Ok(SCmd::Fault { inject: false, .. })));
    assert!(parse_step(Kw::Given, "fault FM-1 is cleared").is_err());
}
