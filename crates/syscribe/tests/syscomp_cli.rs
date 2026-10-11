//! TC-TRS-SYSCOMP-001 / REQ-TRS-SYSCOMP-001 (GH #297): `simulate` of a TestCase whose subject is a composite.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const FILES: &[(&str, &str)] = &[
    ("TxM", "---\ntype: StateDef\nname: TxM\nvariables:\n  - {name: level, value: \"7\"}\nsubStates:\n  - name: Idle\n    isInitial: true\n    transitions:\n      - {internal: true, accept: Tick, effect: {send: Cmd, via: out, payload: {n: \"level\"}}}\n---\n"),
    ("RxM", "---\ntype: StateDef\nname: RxM\nvariables:\n  - {name: got, value: \"0\"}\nsubStates:\n  - name: Waiting\n    isInitial: true\n    transitions:\n      - {target: Done, accept: {payload: Cmd, via: inp, as: m}, assign: \"got := m.n\"}\n  - {name: Done, isFinal: true}\n---\n"),
    ("TxP", "---\ntype: PartDef\nname: TxP\nexhibitsStates: [TxM]\nfeatures:\n  - {name: out, type: Port, direction: out}\n---\nTransmitter.\n"),
    ("RxP", "---\ntype: PartDef\nname: RxP\nexhibitsStates: [RxM]\nfeatures:\n  - {name: inp, type: Port, direction: in}\n---\nReceiver.\n"),
    ("Sys", "---\ntype: PartDef\nname: Sys\nfeatures:\n  - {name: tx, typedBy: TxP}\n  - {name: rx, typedBy: RxP}\nconnections:\n  - {from: tx.out, to: rx.inp, delay: \"5 ms\"}\n---\nThe system.\n"),
    ("REQ-SYS-001", "---\ntype: Requirement\nid: REQ-SYS-001\nname: delivery\nstatus: approved\nreqDomain: software\n---\n\nThe receiver shall get the command.\n"),
];

const TC: &str = "---
type: TestCase
id: TC-SYS-001
name: system
status: active
testLevel: L3
subject: Sys
verifies: [REQ-SYS-001]
---

```gherkin
Feature: system

  Scenario: the command arrives after the link delay
    When tx receives Tick
    Then tx sent Cmd via out
    And rx is in state Waiting
    When 6 ms elapses
    Then rx is in state Done
    And rx.got == 7

  Scenario: a lost command is detected
    Then tx did not send Cmd
    And rx.got == 0
```
";

fn model(tc: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-syscli-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
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
fn scenarios_run_on_the_composite_and_report_the_messages() {
    let root = model(TC);
    let (c, t) = run(&root, &["simulate", "TC-SYS-001", "--json"]);
    assert_eq!(c, 0, "{t}");
    let j: serde_json::Value = serde_json::from_str(&t).unwrap();
    let v = j["verdicts"].as_array().unwrap();
    assert!(v.iter().all(|x| x["status"] == "pass"), "{t}");
    let msgs = v[0]["messages"].as_array().unwrap();
    assert!(msgs[0].as_str().unwrap().contains("tx sent Cmd via out") && msgs[0].as_str().unwrap().contains("delivered to rx.inp at 0.005"), "{msgs:?}");
    assert!(v[1].get("messages").is_none(), "no traffic in the second scenario");
    assert_eq!(j["machineHash"].as_str().unwrap().len(), 64);
}

#[test]
fn a_failing_assertion_names_the_step_and_exits_2() {
    let root = model(&TC.replace("When 6 ms elapses", "When 3 ms elapses"));
    let (c, t) = run(&root, &["simulate", "TC-SYS-001"]);
    assert_eq!(c, 2, "{t}");
    assert!(t.contains("expected rx in state Done"), "{t}");
}

#[test]
fn the_composite_can_be_named_instead_of_the_testcase() {
    let (c, t) = run(&model(TC), &["simulate", "Sys"]);
    assert_eq!(c, 0, "{t}");
    assert!(t.contains("2/2 scenario(s) pass"), "{t}");
}

#[test]
fn steps_outside_the_vocabulary_are_e886_and_coverage_is_refused() {
    let root = model(&TC.replace("When tx receives Tick", "When tx is poked"));
    let (c, t) = run(&root, &["validate"]);
    assert_eq!(c, 1, "{t}");
    assert!(t.contains("E886"), "{t}");
    let (c, t) = run(&model(TC), &["simulate", "TC-SYS-001", "--coverage"]);
    assert_eq!(c, 1, "{t}");
    assert!(t.contains("coverage"), "{t}");
}

#[test]
fn record_keeps_the_messages_beside_the_trace() {
    let root = model(TC);
    let (c, t) = run(&root, &["simulate", "TC-SYS-001", "--record", "--run", "r1"]);
    assert_eq!(c, 0, "{t}");
    let any = std::fs::read_dir(root.join(".syscribe/traces")).unwrap().map(|e| e.unwrap().path()).find(|p| std::fs::read_to_string(p).unwrap().contains("\"messages\"")).expect("a trace with messages");
    let j: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(any).unwrap()).unwrap();
    assert!(j["messages"][0].as_str().unwrap().contains("tx sent Cmd"));
}
