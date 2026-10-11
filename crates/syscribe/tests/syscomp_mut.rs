//! TC-TRS-SYSMUT-001 / REQ-TRS-SYSMUT-001 (GH #299): mutation analysis of the parts of a composite.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn files(extra_scenarios: &str) -> Vec<(String, String)> {
    let f = |n: &str, c: &str| (n.to_string(), c.to_string());
    vec![
        f("TxM", "---\ntype: StateDef\nname: TxM\nvariables:\n  - {name: level, value: \"7\"}\nsubStates:\n  - name: Idle\n    isInitial: true\n    transitions:\n      - {target: Idle, accept: Tick, effect: {send: Cmd, via: out, payload: {n: \"level\"}}}\n---\n"),
        f("RxM", "---\ntype: StateDef\nname: RxM\nvariables:\n  - {name: got, value: \"0\"}\nsubStates:\n  - name: Waiting\n    isInitial: true\n    transitions:\n      - {target: Done, accept: {payload: Cmd, via: inp, as: m}, guard: \"m.n > 5\", assign: \"got := m.n\"}\n  - {name: Done, isFinal: true}\n---\n"),
        f("TxP", "---\ntype: PartDef\nname: TxP\nexhibitsStates: [TxM]\nfeatures:\n  - {name: out, type: Port, direction: out}\n---\nTransmitter.\n"),
        f("RxP", "---\ntype: PartDef\nname: RxP\nexhibitsStates: [RxM]\nfeatures:\n  - {name: inp, type: Port, direction: in}\n---\nReceiver.\n"),
        f("Sys", "---\ntype: PartDef\nname: Sys\nfeatures:\n  - {name: tx, typedBy: TxP}\n  - {name: rx, typedBy: RxP}\nconnections:\n  - {from: tx.out, to: rx.inp}\n---\nThe system.\n"),
        f("REQ-SYS-001", "---\ntype: Requirement\nid: REQ-SYS-001\nname: delivery\nstatus: approved\nreqDomain: software\n---\n\nThe receiver shall get the command.\n"),
        f(
            "TC-SYS-001",
            &format!("---\ntype: TestCase\nid: TC-SYS-001\nname: system\nstatus: active\ntestLevel: L3\nsubject: Sys\nverifies: [REQ-SYS-001]\n---\n\n```gherkin\nFeature: system\n\n  Scenario: delivered\n    When tx receives Tick\n    Then rx is in state Done\n    And rx.got == 7\n\n  Scenario: nothing sent\n    Then rx is in state Waiting\n{extra_scenarios}```\n"),
        ),
    ]
}

fn model(extra: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-sysmut-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    for (n, c) in files(extra) {
        std::fs::write(root.join(format!("{n}.md")), c).unwrap();
    }
    root
}

/// A scenario that distinguishes `m.n > 5` from `m.n >= 5`: level 5 must not be accepted.
const BOUNDARY: &str = "\n  Scenario: a command at the boundary is rejected\n    Given tx.level is 5\n    When tx receives Tick\n    Then rx is in state Waiting\n";

fn run(root: &PathBuf, args: &[&str]) -> (i32, String, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (o.status.code().unwrap_or(-1), String::from_utf8_lossy(&o.stdout).to_string(), String::from_utf8_lossy(&o.stderr).to_string())
}

fn part<'a>(j: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    j["parts"].as_array().unwrap().iter().find(|p| p["part"] == name).unwrap_or_else(|| panic!("no part {name}: {j}"))
}

#[test]
fn a_weak_scenario_set_lets_a_mutant_of_one_part_survive() {
    let (c, t, e) = run(&model(""), &["mutate", "Sys", "--json"]);
    assert_eq!(c, 0, "{t}{e}");
    let j: serde_json::Value = serde_json::from_str(&t).unwrap();
    assert_eq!(j["composite"], "Sys");
    let rx = part(&j, "rx");
    let survivors: Vec<&str> = rx["mutants"].as_array().unwrap().iter().filter(|m| m["fate"] == "survived").map(|m| m["description"].as_str().unwrap()).collect();
    assert!(survivors.iter().any(|d| d.contains("m.n > 5")), "the boundary mutant survives: {survivors:?}");
    assert!(rx["score"].as_f64().unwrap() < 100.0, "{rx}");
    assert!(rx["basis"].is_null() && rx["machine"].as_str().unwrap().contains("Sys.rx"), "{rx}");
    assert!(j["score"].as_f64().unwrap() <= 100.0);
}

#[test]
fn a_scenario_at_the_boundary_kills_it() {
    let (c, t, e) = run(&model(BOUNDARY), &["mutate", "Sys", "--json", "--part", "rx"]);
    assert_eq!(c, 0, "{t}{e}");
    let j: serde_json::Value = serde_json::from_str(&t).unwrap();
    assert_eq!(j["parts"].as_array().unwrap().len(), 1, "--part restricts the analysis");
    let rx = part(&j, "rx");
    let alive: Vec<&str> = rx["mutants"].as_array().unwrap().iter().filter(|m| m["fate"] == "survived").map(|m| m["description"].as_str().unwrap()).collect();
    assert!(!alive.iter().any(|d| d.contains("m.n > 5") && d.contains(">=")), "the boundary mutant is killed: {alive:?}");
}

#[test]
fn the_text_report_names_parts_and_the_gate_names_the_one_below() {
    let (c, t, _) = run(&model(""), &["mutate", "TC-SYS-001"]);
    assert_eq!(c, 0, "{t}");
    assert!(t.contains("Sys.rx") && t.contains("SURVIVED"), "{t}");
    let (c, _t, e) = run(&model(""), &["mutate", "Sys", "--min-score", "100"]);
    assert_eq!(c, 2, "{e}");
    assert!(e.contains("Sys.rx") && e.contains("below"), "{e}");
}

#[test]
fn scenarios_that_fail_on_the_unmutated_composite_stop_the_analysis() {
    let broken = "\n  Scenario: wrong\n    Then rx is in state Done\n";
    let (c, _t, e) = run(&model(broken), &["mutate", "Sys"]);
    assert_eq!(c, 2, "{e}");
    assert!(e.contains("does not pass on the unmutated"), "{e}");
}

#[test]
fn a_part_the_scenarios_never_exercise_fails_the_gate_instead_of_scoring_100() {
    // only the "nothing sent" scenario: rx never moves, tx never ticks
    let root = model("");
    let tc = std::fs::read_to_string(root.join("TC-SYS-001.md")).unwrap();
    let only_idle = tc.replace("  Scenario: delivered\n    When tx receives Tick\n    Then rx is in state Done\n    And rx.got == 7\n\n", "");
    std::fs::write(root.join("TC-SYS-001.md"), only_idle).unwrap();
    let (c, t, e) = run(&root, &["mutate", "Sys", "--min-score", "100"]);
    assert_eq!(c, 2, "{t}{e}");
    assert!(e.contains("no mutant judged") && e.contains("Sys."), "{e}");
    assert!(t.contains("note: no mutant of this part was judged"), "{t}");
    let (c, _, e) = run(&root, &["mutate", "Sys"]);
    assert_eq!(c, 0, "no gate, no failure: {e}");
}

#[test]
fn usage_errors_exit_1_before_any_scenario_runs() {
    let root = model("");
    let (c, _, e) = run(&root, &["mutate", "Sys", "--part", "nope"]);
    assert_eq!(c, 1, "{e}");
    assert!(e.contains("parts: ") && e.contains("rx"), "{e}");
    let (c, _, e) = run(&root, &["mutate", "TxP"]);
    assert_eq!(c, 1, "a part that is not an executable composite: {e}");
    let (c, _, e) = run(&root, &["mutate", "TxM", "--part", "tx"]);
    assert_eq!(c, 1, "--part needs a composite: {e}");
}
