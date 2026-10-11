//! TC-TRS-SYSCOV-001 / REQ-TRS-SYSCOV-001 (GH #299): coverage of composite scenarios, per part and in aggregate.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const FILES: &[(&str, &str)] = &[
    ("TxM", "---\ntype: StateDef\nname: TxM\nvariables:\n  - {name: level, value: \"7\"}\nsubStates:\n  - name: Idle\n    isInitial: true\n    transitions:\n      - {target: Idle, accept: Tick, effect: {send: Cmd, via: out, payload: {n: \"level\"}}}\n---\n"),
    ("RxM", "---\ntype: StateDef\nname: RxM\nvariables:\n  - {name: got, value: \"0\"}\nsubStates:\n  - name: Waiting\n    isInitial: true\n    transitions:\n      - {target: Done, accept: {payload: Cmd, via: inp, as: m}, assign: \"got := m.n\"}\n  - {name: Done, isFinal: true}\n---\n"),
    ("TxP", "---\ntype: PartDef\nname: TxP\nexhibitsStates: [TxM]\nfeatures:\n  - {name: out, type: Port, direction: out}\n---\nTransmitter.\n"),
    ("RxP", "---\ntype: PartDef\nname: RxP\nexhibitsStates: [RxM]\nfeatures:\n  - {name: inp, type: Port, direction: in}\n---\nReceiver.\n"),
    ("Sys", "---\ntype: PartDef\nname: Sys\nfeatures:\n  - {name: tx, typedBy: TxP}\n  - {name: rx, typedBy: RxP}\nconnections:\n  - {from: tx.out, to: rx.inp}\n---\nThe system.\n"),
    ("REQ-SYS-001", "---\ntype: Requirement\nid: REQ-SYS-001\nname: delivery\nstatus: approved\nreqDomain: software\n---\n\nThe receiver shall get the command.\n"),
    ("TC-SYS-001", "---\ntype: TestCase\nid: TC-SYS-001\nname: system\nstatus: active\ntestLevel: L3\nsubject: Sys\nverifies: [REQ-SYS-001]\n---\n\n```gherkin\nFeature: system\n\n  Scenario: delivered\n    When tx receives Tick\n    Then rx is in state Done\n    And rx.got == 7\n\n  Scenario: nothing sent\n    Then rx is in state Waiting\n```\n"),
];

fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-syscov-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    for (n, c) in FILES {
        std::fs::write(root.join(format!("{n}.md")), c).unwrap();
    }
    root
}

fn run(root: &PathBuf, args: &[&str]) -> (i32, String, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (o.status.code().unwrap_or(-1), String::from_utf8_lossy(&o.stdout).to_string(), String::from_utf8_lossy(&o.stderr).to_string())
}

fn part<'a>(j: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    j["coverage"]["parts"].as_array().unwrap().iter().find(|p| p["part"] == name).unwrap_or_else(|| panic!("no part {name}: {j}"))
}

#[test]
fn each_part_has_a_report_and_the_aggregate_sums_them() {
    let (c, t, e) = run(&model(), &["simulate", "TC-SYS-001", "--coverage", "--json"]);
    assert_eq!(c, 0, "{t}{e}");
    let j: serde_json::Value = serde_json::from_str(&t).unwrap();
    assert_eq!(j["coverage"]["composite"], "Sys");
    let (tx, rx) = (part(&j, "tx"), part(&j, "rx"));
    assert!(tx["basis"]["machine"].as_str().unwrap().contains("Sys.tx"), "{tx}");
    assert_eq!((tx["states"]["covered"].as_u64(), tx["states"]["total"].as_u64()), (Some(1), Some(1)));
    assert_eq!((rx["states"]["covered"].as_u64(), rx["states"]["total"].as_u64()), (Some(2), Some(2)));
    let agg = &j["coverage"]["aggregate"];
    assert_eq!((agg["states"]["covered"].as_u64(), agg["states"]["total"].as_u64()), (Some(3), Some(3)));
    assert_eq!((agg["transitions"]["covered"].as_u64(), agg["transitions"]["total"].as_u64()), (Some(2), Some(2)));
}

#[test]
fn fewer_scenarios_cover_less_and_the_text_names_the_part() {
    let (c, t, e) = run(&model(), &["simulate", "TC-SYS-001", "--coverage", "--scenario", "nothing sent"]);
    assert_eq!(c, 0, "{t}{e}");
    assert!(t.contains("coverage of Sys.rx"), "{t}");
    assert!(t.contains("state not entered: Done"), "{t}");
    assert!(t.contains("aggregate"), "{t}");
}

#[test]
fn a_gate_applies_to_every_part_and_names_the_one_below() {
    let (c, _t, e) = run(&model(), &["simulate", "TC-SYS-001", "--coverage", "--scenario", "nothing sent", "--min-states", "100"]);
    assert_eq!(c, 2, "{e}");
    assert!(e.contains("Sys.rx") && e.contains("states"), "{e}");
    let (c, _t, e) = run(&model(), &["simulate", "TC-SYS-001", "--coverage", "--min-states", "100"]);
    assert_eq!(c, 0, "{e}");
}
