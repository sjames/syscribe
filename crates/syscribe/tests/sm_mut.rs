//! TC-TRS-SMMUT-001 / REQ-TRS-SMMUT-001 (GH #282): mutation analysis of state-machine scenarios.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const MACHINE: &str = "variables:
  - {name: lvl, value: \"0\"}
subStates:
  - name: A
    isInitial: true
    transitions:
      - {target: B, accept: Cmd::Go, guard: \"lvl > 2\"}
  - name: B
    transitions:
      - {target: A, accept: Cmd::Back, assign: \"lvl := 0\"}
      - {target: C, accept: {after: \"4 s\"}}
  - name: C
";

const S1: &str = "  Scenario: go
    Given M is in state A
    And lvl is 3
    When Cmd::Go is injected
    Then M is in state B
";
const REST: &str = "
  Scenario: guard boundary
    Given M is in state A
    And lvl is 2
    When Cmd::Go is injected
    Then M is in state A

  Scenario: back resets
    Given M is in state B
    And lvl is 5
    When Cmd::Back is injected
    Then M is in state A
    And lvl == 0

  Scenario: timeout
    Given M is in state B
    When 4 s elapse
    Then M is in state C

  Scenario: not yet
    Given M is in state B
    When 3 s elapse
    Then M is in state B
";

fn model(scenarios: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-smmut-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(root.join("M.md"), format!("---\ntype: StateDef\nname: M\n{MACHINE}---\n")).unwrap();
    std::fs::write(
        root.join("TC-DOOR-001.md"),
        format!("---\ntype: TestCase\nid: TC-DOOR-001\nname: Door\nstatus: active\ntestLevel: L2\nsubject: M\n---\n\n```gherkin\nFeature: door\n\n{scenarios}```\n"),
    )
    .unwrap();
    root
}

fn mutate(scenarios: &str, extra: &[&str]) -> (i32, serde_json::Value, String) {
    let root = model(scenarios);
    let before = std::fs::read_to_string(root.join("M.md")).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&root).args(["mutate", "M", "--json"]).args(extra).output().unwrap();
    assert_eq!(before, std::fs::read_to_string(root.join("M.md")).unwrap(), "model files are never modified");
    let text = format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr));
    (o.status.code().unwrap_or(-1), serde_json::from_slice(&o.stdout).unwrap_or(serde_json::Value::Null), text)
}

fn ops(j: &serde_json::Value) -> Vec<String> {
    j["mutants"].as_array().unwrap().iter().map(|m| m["operator"].as_str().unwrap().to_string()).collect()
}

#[test]
fn a_thorough_scenario_set_kills_every_mutant() {
    let (c, j, t) = mutate(&format!("{S1}{REST}"), &[]);
    assert_eq!(c, 0, "{t}");
    assert_eq!(j["survived"], 0, "{t}");
    assert_eq!(j["score"].as_f64().unwrap(), 100.0);
    let o = ops(&j);
    for want in ["negate-condition", "relational-operator", "literal-shift", "retarget", "remove-transition", "remove-action", "assign-self", "scale-after"] {
        assert!(o.iter().any(|x| x == want), "operator {want} not generated: {o:?}");
    }
    assert_eq!(j["mutants"][0]["origin"]["qname"], "M");
}

#[test]
fn a_weak_scenario_set_leaves_survivors_with_the_mutation_shown() {
    let (c, j, t) = mutate(S1, &[]);
    assert_eq!(c, 0, "{t}");
    assert!(j["survived"].as_u64().unwrap() > 0, "{t}");
    assert!(j["score"].as_f64().unwrap() < 100.0);
    let surv: Vec<_> = j["mutants"].as_array().unwrap().iter().filter(|m| m["fate"] == "survived").collect();
    assert!(surv.iter().any(|m| m["description"].as_str().unwrap().contains(">=")), "the >= mutant survives lvl=3: {t}");
    assert!(surv.iter().all(|m| m["origin"]["path"].as_str().is_some()));
}

#[test]
fn mutants_in_uncovered_code_with_an_identical_trace_are_equivalent_suspects_not_survivors() {
    let (_, j, t) = mutate(S1, &[]);
    assert!(j["equivalentSuspect"].as_u64().unwrap() >= 1, "{t}");
    let counted = j["killed"].as_u64().unwrap() + j["survived"].as_u64().unwrap();
    let score = 100.0 * j["killed"].as_u64().unwrap() as f64 / counted as f64;
    assert!((j["score"].as_f64().unwrap() - score).abs() < 1e-6, "score = killed / (killed + survived)");
}

#[test]
fn min_score_gates_the_exit_code() {
    assert_eq!(mutate(S1, &["--min-score", "95"]).0, 2);
    assert_eq!(mutate(&format!("{S1}{REST}"), &["--min-score", "95"]).0, 0);
}

#[test]
fn scenarios_must_pass_on_the_unmutated_machine() {
    let (c, _, t) = mutate("  Scenario: wrong\n    Given M is in state A\n    Then M is in state C\n", &[]);
    assert_eq!(c, 2, "{t}");
    assert!(t.contains("unmutated"), "{t}");
}

#[test]
fn no_scenarios_is_a_usage_error() {
    let root = model("");
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&root).args(["mutate", "M"]).output().unwrap();
    assert_eq!(o.status.code(), Some(1));
}
