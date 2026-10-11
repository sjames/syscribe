//! TC-TRS-SMCOV-001 / REQ-TRS-SMCOV-001 (GH #281): measured coverage of state-machine scenarios.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const MACHINE: &str = "variables:
  - {name: x, value: \"0\"}
  - {name: y, value: \"false\"}
subStates:
  - name: A
    isInitial: true
    transitions:
      - {target: B, accept: Cmd::Go, guard: \"x > 1 and y\"}
      - {target: T, accept: {after: \"5 s\"}}
  - name: B
    transitions:
      - {target: A, accept: Cmd::Back}
  - name: T
  - name: C
";

const S1: &str = "  Scenario: go
    Given M is in state A
    And x is 2
    And y is true
    When Cmd::Go is injected
    Then M is in state B
    When Cmd::Back is injected
    Then M is in state A
";
const S2: &str = "  Scenario: guard blocks
    Given M is in state A
    And x is 0
    And y is true
    When Cmd::Go is injected
    Then M is in state A
";
const S3: &str = "  Scenario: timeout
    Given M is in state A
    When 5 s elapse
    Then M is in state T
";

fn model(scenarios: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-smcov-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(root.join("M.md"), format!("---\ntype: StateDef\nname: M\n{MACHINE}---\n")).unwrap();
    std::fs::write(root.join("REQ-DOOR-001.md"), "---\ntype: Requirement\nid: REQ-DOOR-001\nname: door\nstatus: approved\nreqDomain: software\n---\n\nThe door shall work.\n").unwrap();
    std::fs::write(root.join("REQ-DOOR-002.md"), "---\ntype: Requirement\nid: REQ-DOOR-002\nname: other\nstatus: approved\nreqDomain: software\n---\n\nThe door shall also work.\n").unwrap();
    std::fs::write(
        root.join("TC-DOOR-001.md"),
        format!("---\ntype: TestCase\nid: TC-DOOR-001\nname: Door\nstatus: active\ntestLevel: L2\nsubject: M\nverifies: [REQ-DOOR-001]\n---\n\n```gherkin\nFeature: door\n\n{scenarios}```\n"),
    )
    .unwrap();
    root
}

fn cov(scenarios: &str, extra: &[&str]) -> (i32, serde_json::Value, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(model(scenarios)).args(["simulate", "M", "--all", "--coverage", "--json"]).args(extra).output().unwrap();
    let text = format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr));
    let j = serde_json::from_slice(&o.stdout).unwrap_or(serde_json::Value::Null);
    (o.status.code().unwrap_or(-1), j, text)
}

#[test]
fn states_transitions_and_pairs() {
    let (c, j, t) = cov(&format!("{S1}\n{S2}"), &[]);
    assert_eq!(c, 0, "{t}");
    let r = &j["coverage"];
    assert_eq!((r["states"]["covered"].as_u64(), r["states"]["total"].as_u64()), (Some(2), Some(4)), "{t}");
    assert!(r["states"]["uncovered"].as_array().unwrap().iter().any(|s| s == "C"));
    assert_eq!((r["transitions"]["covered"].as_u64(), r["transitions"]["total"].as_u64()), (Some(2), Some(3)));
    assert!(r["transitions"]["uncovered"][0]["description"].as_str().unwrap().contains("A → T"), "{t}");
    assert_eq!((r["pairs"]["covered"].as_u64(), r["pairs"]["total"].as_u64()), (Some(1), Some(3)));
}

#[test]
fn guard_conditions_and_mcdc() {
    let (_, j, t) = cov(&format!("{S1}\n{S2}"), &[]);
    let g = &j["coverage"]["guards"];
    assert_eq!((g["covered"].as_u64(), g["total"].as_u64()), (Some(3), Some(4)), "x seen both, y only true: {t}");
    let e = &g["expressions"][0];
    assert_eq!(e["atoms"][0]["seenTrue"], true);
    assert_eq!(e["atoms"][0]["seenFalse"], true);
    assert_eq!(e["atoms"][1]["seenFalse"], false);
    assert_eq!(e["mcdc"][0]["independent"], true, "(T,T)→T and (F,T)→F differ only in x");
    assert_eq!(e["mcdc"][1]["independent"], false);
}

#[test]
fn timers_must_both_fire_and_be_cancelled() {
    let (_, j, t) = cov(&format!("{S1}\n{S2}"), &[]);
    assert_eq!(j["coverage"]["timers"]["covered"], 0, "cancelled by the exit of A in `go`, never fired: {t}");
    assert_eq!(j["coverage"]["timers"]["total"], 1);
    let (_, j, t) = cov(&format!("{S1}\n{S2}\n{S3}"), &[]);
    assert_eq!(j["coverage"]["timers"]["covered"], 1, "{t}");
    assert_eq!(j["coverage"]["transitions"]["covered"], 3);
}

#[test]
fn requirement_coverage_follows_passing_scenarios() {
    let (_, j, t) = cov(&format!("{S1}\n{S2}"), &[]);
    assert_eq!(j["coverage"]["requirements"]["covered"][0], "REQ-DOOR-001", "{t}");
    let failing = format!("{S1}\n  Scenario: broken\n    Given M is in state A\n    Then M is in state C\n");
    let (c, j, _) = cov(&failing, &[]);
    assert_eq!(c, 2);
    assert_eq!(j["coverage"]["requirements"]["covered"].as_array().unwrap().len(), 0, "a TestCase with a failing scenario verifies nothing");
}

#[test]
fn thresholds_gate_the_exit_code() {
    let (c, _, _) = cov(&format!("{S1}\n{S2}"), &["--min-states", "50"]);
    assert_eq!(c, 0, "2/4 = 50%");
    let (c, _, t) = cov(&format!("{S1}\n{S2}"), &["--min-states", "51"]);
    assert_eq!(c, 2, "{t}");
    let (c, _, _) = cov(&format!("{S1}\n{S2}"), &["--min-guards", "80"]);
    assert_eq!(c, 2, "3/4 = 75%");
    let (c, _, _) = cov(&format!("{S1}\n{S2}"), &["--min-transitions", "66"]);
    assert_eq!(c, 0);
}

#[test]
fn the_report_states_its_basis() {
    let (_, j, _) = cov(&format!("{S1}\n{S2}"), &[]);
    let b = &j["coverage"]["basis"];
    assert_eq!(b["scenarios"], 2);
    assert_eq!(b["machine"], "M");
    assert_eq!(b["machineHash"].as_str().unwrap().len(), 64);
    assert!(b["toolVersion"].as_str().is_some_and(|v| !v.is_empty()));
}

#[test]
fn a_short_circuited_atom_still_shows_its_other_values() {
    let machine = MACHINE.replace("x > 1 and y", "x != 0 and 10 / x > 1");
    static N: AtomicU64 = AtomicU64::new(5000);
    let root = std::env::temp_dir().join(format!("syscribe-smcov-sc-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(root.join("M.md"), format!("---\ntype: StateDef\nname: M\n{machine}---\n")).unwrap();
    std::fs::write(
        root.join("TC-DOOR-001.md"),
        "---\ntype: TestCase\nid: TC-DOOR-001\nname: Door\nstatus: active\ntestLevel: L2\nsubject: M\n---\n\n```gherkin\nFeature: f\n\n  Scenario: zero\n    Given M is in state A\n    And x is 0\n    When Cmd::Go is injected\n    Then M is in state A\n\n  Scenario: five\n    Given M is in state A\n    And x is 5\n    When Cmd::Go is injected\n    Then M is in state B\n```\n",
    )
    .unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&root).args(["simulate", "M", "--all", "--coverage", "--json"]).output().unwrap();
    let j: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap_or_else(|_| panic!("{}", String::from_utf8_lossy(&o.stderr)));
    let atoms = &j["coverage"]["guards"]["expressions"][0]["atoms"];
    assert_eq!(atoms[0]["seenFalse"], true, "x != 0 was false when x = 0 even though the division was short-circuited: {j}");
    assert_eq!(atoms[0]["seenTrue"], true);
}
