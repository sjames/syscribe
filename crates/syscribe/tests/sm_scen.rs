//! TC-TRS-SMSCEN-001 / REQ-TRS-SMSCEN-001 (GH #280): executable Gherkin scenarios for state machines.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const MACHINE: &str = "variables:
  - {name: level, value: \"0\"}
subStates:
  - name: Closed
    isInitial: true
    transitions:
      - {target: Opening, accept: Cmd::Open}
  - name: Opening
    transitions:
      - {target: Open, accept: {after: \"3 s\"}}
      - {target: Closed, accept: {when: \"level > 2\"}}
  - name: Open
    transitions:
      - {target: Closed, accept: {payload: Cmd::Close, as: c}, guard: \"c.force < 10\"}
";

fn model(scenarios: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-smscen-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(root.join("M.md"), format!("---\ntype: StateDef\nname: M\n{MACHINE}---\n")).unwrap();
    std::fs::write(
        root.join("TC-DOOR-001.md"),
        format!("---\ntype: TestCase\nid: TC-DOOR-001\nname: Door scenarios\nstatus: active\ntestLevel: L2\nsubject: M\n---\n\n```gherkin\nFeature: door\n\n{scenarios}```\n"),
    )
    .unwrap();
    root
}

fn run(root: &PathBuf, args: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (o.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

const GOOD: &str = "  Scenario: opens after three seconds
    Given M is in state Closed
    When Cmd::Open is injected
    Then M is in state Opening
    When 3 s elapse
    Then M is in state Open

  Scenario: obstruction aborts opening
    Given M is in state Opening
    And level is 0
    When level becomes 3
    Then M is in state Closed
    And level == 3
";

#[test]
fn passing_scenarios_exit_0() {
    let root = model(GOOD);
    let (c, t) = run(&root, &["simulate", "TC-DOOR-001", "--json"]);
    assert_eq!(c, 0, "{t}");
    let j: serde_json::Value = serde_json::from_str(&t).unwrap();
    let v = j["verdicts"].as_array().unwrap();
    assert_eq!(v.len(), 2);
    assert!(v.iter().all(|x| x["status"] == "pass"), "{t}");
    assert_eq!(j["machineHash"].as_str().unwrap().len(), 64);
}

#[test]
fn a_false_then_fails_with_expected_and_actual() {
    let root = model("  Scenario: wrong\n    Given M is in state Closed\n    When Cmd::Open is injected\n    Then M is in state Open\n");
    let (c, t) = run(&root, &["simulate", "TC-DOOR-001", "--json"]);
    assert_eq!(c, 2, "{t}");
    let j: serde_json::Value = serde_json::from_str(&t).unwrap();
    assert_eq!(j["verdicts"][0]["status"], "fail");
    let msg = j["verdicts"][0]["message"].as_str().unwrap();
    assert!(msg.contains("Open") && msg.contains("Opening"), "{msg}");
}

#[test]
fn errors_are_distinct_from_failures() {
    let root = model("  Scenario: bad state\n    Given M is in state Nowhere\n    Then M is in state Closed\n\n  Scenario: run error\n    Given M is in state Open\n    When Cmd::Close is injected\n");
    let (c, t) = run(&root, &["simulate", "TC-DOOR-001", "--json"]);
    assert_eq!(c, 2, "{t}");
    let j: serde_json::Value = serde_json::from_str(&t).unwrap();
    assert_eq!(j["verdicts"][0]["status"], "error");
    assert_eq!(j["verdicts"][1]["status"], "error", "missing payload field c.force stops the run: {t}");
}

#[test]
fn outlines_expand_one_scenario_per_row() {
    let root = model(
        "  Scenario Outline: obstruction <n>
    Given M is in state Opening
    When level becomes <n>
    Then M is in state <state>

    Examples:
      | n | state   |
      | 1 | Opening |
      | 3 | Closed  |
",
    );
    let (c, t) = run(&root, &["simulate", "TC-DOOR-001", "--json"]);
    assert_eq!(c, 0, "{t}");
    let j: serde_json::Value = serde_json::from_str(&t).unwrap();
    let titles: Vec<_> = j["verdicts"].as_array().unwrap().iter().map(|v| v["scenario"].as_str().unwrap().to_string()).collect();
    assert_eq!(titles, ["obstruction <n> [row 1]", "obstruction <n> [row 2]"]);
}

#[test]
fn a_step_outside_the_vocabulary_is_e886_and_never_skipped() {
    let root = model("  Scenario: prose\n    Given the door is stuck\n    When Cmd::Open is injected\n    Then the door was not opened at all\n");
    let (_, v) = run(&root, &["validate"]);
    assert!(v.contains("E886"), "{v}");
    let (c, t) = run(&root, &["simulate", "TC-DOOR-001", "--json"]);
    assert_eq!(c, 2, "{t}");
    let j: serde_json::Value = serde_json::from_str(&t).unwrap();
    assert_eq!(j["verdicts"][0]["status"], "error");
}

#[test]
fn the_machine_argument_runs_every_scenario_testcase_and_scenario_filters() {
    let root = model(GOOD);
    let (c, t) = run(&root, &["simulate", "M", "--json"]);
    assert_eq!(c, 0, "{t}");
    let j: serde_json::Value = serde_json::from_str(&t).unwrap();
    assert_eq!(j["verdicts"].as_array().unwrap().len(), 2);
    let (c, t) = run(&root, &["simulate", "TC-DOOR-001", "--scenario", "opens after three seconds", "--json"]);
    assert_eq!(c, 0, "{t}");
    let j: serde_json::Value = serde_json::from_str(&t).unwrap();
    assert_eq!(j["verdicts"].as_array().unwrap().len(), 1);
    let (c, _) = run(&root, &["simulate", "TC-DOOR-001", "--scenario", "no such"]);
    assert_eq!(c, 1, "an unknown scenario is a usage error");
}

#[test]
fn record_writes_verdicts_into_the_results_sidecar_and_the_trace() {
    let root = model(GOOD);
    let (c, t) = run(&root, &["simulate", "TC-DOOR-001", "--record", "--run", "r1"]);
    assert_eq!(c, 0, "{t}");
    let sidecar = std::fs::read_to_string(root.join(".syscribe/results.json")).unwrap();
    assert!(sidecar.contains("TC-DOOR-001::opens after three seconds"), "{sidecar}");
    assert!(sidecar.contains("TC-DOOR-001::obstruction aborts opening"), "{sidecar}");
    assert!(root.join(".syscribe/traces").is_dir());
    assert!(std::fs::read_dir(root.join(".syscribe/traces")).unwrap().count() >= 2);
    assert!(root.join(".syscribe/run-history.json").exists() || std::fs::read_dir(root.join(".syscribe")).unwrap().count() >= 3);
}

#[test]
fn record_keeps_other_scenario_results() {
    let root = model(GOOD);
    std::fs::create_dir_all(root.join(".syscribe")).unwrap();
    let manual = r#"[{"testCase":"TC-OTHER-001","scenario":"manual","steps":["x"],"result":"pass","timestamp":"2026-01-01T00:00:00Z"}]"#;
    std::fs::write(root.join("manual.json"), manual).unwrap();
    let (c, t) = run(&root, &["ingest-results", "--format", "session-log", root.join("manual.json").to_str().unwrap()]);
    assert_eq!(c, 0, "{t}");
    let (c, t) = run(&root, &["simulate", "TC-DOOR-001", "--record"]);
    assert_eq!(c, 0, "{t}");
    let sidecar = std::fs::read_to_string(root.join(".syscribe/results.json")).unwrap();
    assert!(sidecar.contains("TC-OTHER-001::manual"), "a recorded run must not erase manual session-log verdicts: {sidecar}");
}

#[test]
fn a_failing_scenario_is_recorded_as_a_failure() {
    let root = model("  Scenario: wrong\n    Given M is in state Closed\n    Then M is in state Open\n");
    let (c, _) = run(&root, &["simulate", "TC-DOOR-001", "--record"]);
    assert_eq!(c, 2);
    let sidecar = std::fs::read_to_string(root.join(".syscribe/results.json")).unwrap();
    let j: serde_json::Value = serde_json::from_str(&sidecar).unwrap();
    assert_eq!(j["by_scenario"]["TC-DOOR-001::wrong"].as_str().map(|s| s.to_lowercase()), Some("fail".into()), "{sidecar}");
}

#[test]
fn given_a_nested_state_enters_its_ancestors_and_arms_their_timers() {
    static N: AtomicU64 = AtomicU64::new(1000);
    let root = std::env::temp_dir().join(format!("syscribe-smscen-n-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(
        root.join("M.md"),
        "---\ntype: StateDef\nname: M\nsubStates:\n  - name: Outer\n    isInitial: true\n    transitions: [{target: Done, accept: {after: \"2 s\"}}]\n    subStates: [{name: A, isInitial: true}, {name: B}]\n  - name: Done\n---\n",
    )
    .unwrap();
    std::fs::write(
        root.join("TC-N-001.md"),
        "---\ntype: TestCase\nid: TC-N-001\nname: nested\nstatus: active\ntestLevel: L2\nsubject: M\n---\n\n```gherkin\nFeature: f\n\n  Scenario: nested given\n    Given M is in state Outer/B\n    Then M is in state Outer\n    When 2 s elapse\n    Then M is in state Done\n```\n",
    )
    .unwrap();
    let (c, t) = run(&root, &["simulate", "TC-N-001", "--json"]);
    assert_eq!(c, 0, "{t}");
}

const OUTLINE: &str = "  Scenario Outline: obstruction <n>
    Given M is in state Opening
    When level becomes <n>
    Then M is in state <state>

    Examples:
      | n | state   |
      | 1 | Opening |
      | 3 | Closed  |
";

#[test]
fn recording_one_row_of_an_outline_keeps_the_outline_verdict_consistent() {
    let root = model(&OUTLINE.replace("| 3 | Closed  |", "| 3 | Opening |"));
    // row 2 is wrong (3 closes the door) and fails
    let (c, _) = run(&root, &["simulate", "TC-DOOR-001", "--record"]);
    assert_eq!(c, 2);
    // re-running only the passing row must not turn the outline green
    let (c, _) = run(&root, &["simulate", "TC-DOOR-001", "--scenario", "obstruction <n> [row 1]", "--record"]);
    assert_eq!(c, 0);
    let j: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(root.join(".syscribe/results.json")).unwrap()).unwrap();
    let v = |k: &str| j["by_scenario"][k].as_str().unwrap().to_lowercase();
    assert_eq!(v("TC-DOOR-001::obstruction <n> [row 2]"), "fail");
    assert_eq!(v("TC-DOOR-001::obstruction <n>"), "fail", "{j}");
}

#[test]
fn crosscheck_and_thresholds_have_clear_preconditions() {
    let root = model(GOOD);
    assert_eq!(run(&root, &["simulate", "TC-DOOR-001", "--crosscheck", "true"]).0, 1);
    assert_eq!(run(&root, &["simulate", "TC-DOOR-001", "--min-states", "50"]).0, 1);
}
