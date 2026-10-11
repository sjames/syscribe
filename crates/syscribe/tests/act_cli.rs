//! TC-TRS-ACTEXEC-001 / REQ-TRS-ACTEXEC-001 (GH #276): simulate, scenarios, coverage and evidence for action flows.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const A: &str = "---
type: ActionDef
name: A
parameters:
  - {name: n, direction: in}
  - {name: result, direction: out}
variables:
  - {name: acc, value: \"0\"}
subActions:
  - {name: init, kind: AssignmentAction, assign: \"acc := n\"}
  - {name: dbl, kind: AssignmentAction, assign: \"acc := acc * 3\"}
  - {name: big, kind: AssignmentAction, assign: \"result := acc\"}
  - {name: small, kind: AssignmentAction, assign: \"result := 0 - 1\"}
controlNodes:
  - {name: d, kind: DecisionNode}
successionConnections:
  - {after: init, before: dbl}
  - {after: dbl, before: d}
  - {after: d, before: big, guard: \"acc > 10 and n > 0\"}
  - {after: d, before: small, guard: else}
---
";

const W: &str = "---
type: ActionDef
name: W
variables:
  - {name: got, value: \"0\"}
subActions:
  - {name: wait, kind: AcceptAction, payload: Pkg::Go, as: g}
  - {name: take, kind: AssignmentAction, assign: \"got := g.v\"}
successionConnections:
  - {after: wait, before: take}
---
";

fn tc(subject: &str, scenarios: &str) -> String {
    format!("---\ntype: TestCase\nid: TC-ACT-001\nname: scenarios\nstatus: active\ntestLevel: L2\nsubject: {subject}\nverifies: [REQ-ACT-001]\n---\n\n```gherkin\nFeature: f\n\n{scenarios}```\n")
}

fn model(files: &[(&str, &str)]) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-actcli-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(root.join("REQ-ACT-001.md"), "---\ntype: Requirement\nid: REQ-ACT-001\nname: act\nstatus: approved\nreqDomain: software\n---\n\nThe action shall work.\n").unwrap();
    for (n, c) in files {
        std::fs::write(root.join(format!("{n}.md")), c).unwrap();
    }
    root
}

fn run(root: &PathBuf, args: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (o.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

const BIG: &str = "  Scenario: big branch
    Given n is 5
    When the action runs
    Then result == 15
    And big ran
    And small did not run
    And the action completed
";
const SMALL: &str = "  Scenario: small branch
    Given n is 2
    When the action runs
    Then result == -1
    And small ran
    And big did not run
";

#[test]
fn script_mode_runs_the_action_and_prints_outputs() {
    let root = model(&[("A", A)]);
    let (c, t) = run(&root, &["simulate", "A", "--do", "set n=5; run", "--json"]);
    assert_eq!(c, 0, "{t}");
    let j: serde_json::Value = serde_json::from_str(&t).unwrap();
    assert_eq!(j["status"], "completed");
    assert_eq!(j["outputs"]["result"], "15");
    assert_eq!(j["machineHash"].as_str().unwrap().len(), 64);
    let (c, t) = run(&root, &["simulate", "A", "--do", "set n=5; run"]);
    assert_eq!(c, 0, "{t}");
    assert!(t.contains("result = 15"), "{t}");
    let (c, _) = run(&root, &["simulate", "A", "--do", "frobnicate"]);
    assert_eq!(c, 1);
    let (c, _) = run(&root, &["simulate", "A", "--do", "run"]);
    assert_eq!(c, 2, "n has no value");
}

#[test]
fn scenarios_pass_fail_and_error_like_state_machine_scenarios() {
    let root = model(&[("A", A), ("TC-ACT-001", &tc("A", &format!("{BIG}\n{SMALL}")))]);
    let (c, t) = run(&root, &["simulate", "TC-ACT-001", "--json"]);
    assert_eq!(c, 0, "{t}");
    let j: serde_json::Value = serde_json::from_str(&t).unwrap();
    assert!(j["verdicts"].as_array().unwrap().iter().all(|v| v["status"] == "pass"), "{t}");
    let wrong = model(&[("A", A), ("TC-ACT-001", &tc("A", "  Scenario: wrong\n    Given n is 5\n    When the action runs\n    Then small ran\n"))]);
    let (c, t) = run(&wrong, &["simulate", "TC-ACT-001", "--json"]);
    assert_eq!(c, 2);
    let j: serde_json::Value = serde_json::from_str(&t).unwrap();
    assert_eq!(j["verdicts"][0]["status"], "fail");
    assert!(j["verdicts"][0]["message"].as_str().unwrap().contains("small"), "{t}");
    let err = model(&[("A", A), ("TC-ACT-001", &tc("A", "  Scenario: unset\n    When the action runs\n"))]);
    let (c, t) = run(&err, &["simulate", "TC-ACT-001", "--json"]);
    assert_eq!(c, 2);
    let j: serde_json::Value = serde_json::from_str(&t).unwrap();
    assert_eq!(j["verdicts"][0]["status"], "error", "n has no value: {t}");
}

#[test]
fn events_and_waiting_in_scenarios() {
    let root = model(&[("W", W), ("TC-ACT-001", &tc("W", "  Scenario: waits\n    When the action runs\n    Then the action is waiting\n    When Pkg::Go is injected with v = 7\n    Then got == 7\n    And the action completed\n"))]);
    let (c, t) = run(&root, &["simulate", "TC-ACT-001", "--json"]);
    assert_eq!(c, 0, "{t}");
}

#[test]
fn steps_outside_the_vocabulary_are_e886() {
    let root = model(&[("A", A), ("TC-ACT-001", &tc("A", "  Scenario: prose\n    Given the tank is full\n    When the action runs\n    Then the pilot is happy\n"))]);
    let (_, v) = run(&root, &["validate"]);
    assert!(v.contains("E886"), "{v}");
}

#[test]
fn a_machine_name_runs_all_its_scenario_testcases_and_records_evidence() {
    let root = model(&[("A", A), ("TC-ACT-001", &tc("A", &format!("{BIG}\n{SMALL}")))]);
    let (c, t) = run(&root, &["simulate", "A", "--record", "--run", "r1"]);
    assert_eq!(c, 0, "{t}");
    let sidecar = std::fs::read_to_string(root.join(".syscribe/results.json")).unwrap();
    assert!(sidecar.contains("TC-ACT-001::big branch") && sidecar.contains("TC-ACT-001::small branch"), "{sidecar}");
}

#[test]
fn coverage_counts_nodes_edges_decision_branches_and_guard_conditions() {
    let one = model(&[("A", A), ("TC-ACT-001", &tc("A", BIG))]);
    let (c, t) = run(&one, &["simulate", "A", "--all", "--coverage", "--json"]);
    assert_eq!(c, 0, "{t}");
    let j: serde_json::Value = serde_json::from_str(&t).unwrap();
    let r = &j["coverage"];
    assert_eq!((r["nodes"]["covered"].as_u64(), r["nodes"]["total"].as_u64()), (Some(4), Some(5)), "{t}");
    assert_eq!(r["nodes"]["uncovered"][0], "small");
    assert_eq!((r["edges"]["covered"].as_u64(), r["edges"]["total"].as_u64()), (Some(3), Some(4)));
    assert_eq!((r["decisions"]["covered"].as_u64(), r["decisions"]["total"].as_u64()), (Some(1), Some(2)));
    // `acc > 10 and n > 0`: both atoms seen true only
    assert_eq!((r["guards"]["covered"].as_u64(), r["guards"]["total"].as_u64()), (Some(2), Some(4)), "{t}");
    assert_eq!(r["requirements"]["covered"][0], "REQ-ACT-001");
    assert_eq!(r["basis"]["machineHash"].as_str().unwrap().len(), 64);

    let both = model(&[("A", A), ("TC-ACT-001", &tc("A", &format!("{BIG}\n{SMALL}")))]);
    let (_, t) = run(&both, &["simulate", "A", "--all", "--coverage", "--json"]);
    let j: serde_json::Value = serde_json::from_str(&t).unwrap();
    let r = &j["coverage"];
    assert_eq!(r["nodes"]["covered"], 5);
    assert_eq!(r["decisions"]["covered"], 2);
    // n = 5: (T,T)→T ; n = 2: acc = 6, (F,T)→F : acc is independent
    assert_eq!(r["guards"]["expressions"][0]["mcdc"][0]["independent"], true, "{t}");
}

#[test]
fn coverage_gates_decide_the_exit_code() {
    let one = model(&[("A", A), ("TC-ACT-001", &tc("A", BIG))]);
    assert_eq!(run(&one, &["simulate", "A", "--all", "--coverage", "--min-nodes", "80"]).0, 0, "4/5 = 80%");
    assert_eq!(run(&one, &["simulate", "A", "--all", "--coverage", "--min-nodes", "81"]).0, 2);
    assert_eq!(run(&one, &["simulate", "A", "--all", "--coverage", "--min-edges", "76"]).0, 2, "3/4 = 75%");
    assert_eq!(run(&one, &["simulate", "A", "--min-nodes", "50"]).0, 1, "needs --coverage");
}

const M1: &str = "---
type: ActionDef
name: M1
parameters:
  - {name: n, direction: in}
  - {name: result, direction: out}
variables:
  - {name: acc, value: \"0\"}
subActions:
  - {name: init, kind: AssignmentAction, assign: \"acc := n\"}
  - {name: big, kind: AssignmentAction, assign: \"result := 1\"}
  - {name: small, kind: AssignmentAction, assign: \"result := 0\"}
controlNodes:
  - {name: d, kind: DecisionNode}
successionConnections:
  - {after: init, before: d}
  - {after: d, before: big, guard: \"acc > 10\"}
  - {after: d, before: small, guard: else}
---
";
const S_BIG: &str = "  Scenario: eleven
    Given n is 11
    When the action runs
    Then result == 1
    And big ran
    And small did not run
";
const S_EDGE: &str = "
  Scenario: ten
    Given n is 10
    When the action runs
    Then result == 0
    And small ran
    And big did not run
";

fn mutate(scenarios: &str, extra: &[&str]) -> (i32, serde_json::Value, String) {
    let root = model(&[("M1", M1), ("TC-ACT-001", &tc("M1", scenarios))]);
    let (c, t) = run(&root, &[&["mutate", "M1", "--json"][..], extra].concat());
    (c, serde_json::from_str(&t).unwrap_or(serde_json::Value::Null), t)
}

#[test]
fn a_thorough_scenario_set_kills_every_action_mutant() {
    let (c, j, t) = mutate(&format!("{S_BIG}{S_EDGE}"), &[]);
    assert_eq!(c, 0, "{t}");
    // Removing or redirecting `init → d` survives by construction: `init` is declared before `d`, so the
    // document-order tie-break runs it first anyway. Mutation analysis exposes this reliance on ordering;
    // every other mutant is killed.
    let survivors: Vec<_> = j["mutants"].as_array().unwrap().iter().filter(|m| m["fate"] == "survived").collect();
    assert!(survivors.iter().all(|m| m["origin"]["path"] == "successionConnections[0]"), "{t}");
    assert_eq!(survivors.len(), 3, "{t}");
    assert!(j["score"].as_f64().unwrap() > 85.0);
    let ops: Vec<_> = j["mutants"].as_array().unwrap().iter().map(|m| m["operator"].as_str().unwrap().to_string()).collect();
    for want in ["negate-condition", "relational-operator", "literal-shift", "remove-succession", "retarget", "swap-branches", "remove-assignment", "assign-self", "remove-action"] {
        assert!(ops.iter().any(|o| o == want), "{want} not generated: {ops:?}");
    }
}

#[test]
fn a_weak_scenario_set_leaves_survivors_and_min_score_gates() {
    let (c, j, t) = mutate(S_BIG, &[]);
    assert_eq!(c, 0, "{t}");
    assert!(j["survived"].as_u64().unwrap() > 0, "{t}");
    let surv: Vec<_> = j["mutants"].as_array().unwrap().iter().filter(|m| m["fate"] == "survived").collect();
    assert!(surv.iter().any(|m| m["description"].as_str().unwrap().contains(">=")), "{t}");
    assert_eq!(mutate(S_BIG, &["--min-score", "99"]).0, 2);
    assert_eq!(mutate(&format!("{S_BIG}{S_EDGE}"), &["--min-score", "85"]).0, 0);
}

#[test]
fn step_parsing_is_not_fooled_by_keywords_and_names_must_exist() {
    let root = model(&[("A", A), ("TC-ACT-001", &tc("A", "  Scenario: typo\n    Given n is 5\n    When the action runs\n    Then Zzz did not run\n"))]);
    let (c, t) = run(&root, &["simulate", "TC-ACT-001", "--json"]);
    assert_eq!(c, 2, "{t}");
    let j: serde_json::Value = serde_json::from_str(&t).unwrap();
    assert_eq!(j["verdicts"][0]["status"], "error", "a node that does not exist is an error, not a vacuous pass: {t}");
}

#[test]
fn compile_and_validate_agree() {
    let callee = "---\ntype: ActionDef\nname: Callee\nsubActions:\n  - {name: p, kind: Action}\n  - {name: q, kind: Action}\nsuccessionConnections:\n  - {after: p, before: q, guard: \"operator confirms\"}\n---\n";
    let top = "---\ntype: ActionDef\nname: Top\nvariables:\n  - {name: v, value: \"0\"}\nsubActions:\n  - {name: c, kind: PerformAction, typedBy: Callee}\n---\n";
    let root = model(&[("Callee", callee), ("Top", top)]);
    let (_, v) = run(&root, &["validate"]);
    assert!(v.contains("W935") && v.contains("Callee"), "validate must see what compile will refuse: {v}");
    let (c, _) = run(&root, &["compile", "Top"]);
    assert_eq!(c, 2);
    let unknown = "---\ntype: ActionDef\nname: U\nvariables:\n  - {name: v, value: \"0\"}\nsubActions:\n  - {name: a, kind: Action}\nsuccessionConnections:\n  - {after: a, before: ghost}\n---\n";
    let (_, v) = run(&model(&[("U", unknown)]), &["validate"]);
    assert!(v.contains("E887") && v.contains("ghost"), "{v}");
    let noc = "---\ntype: ActionDef\nname: N\nvariables:\n  - {name: v, value: \"0\"}\nsubActions:\n  - {name: i, kind: IfAction}\n---\n";
    let (_, v) = run(&model(&[("N", noc)]), &["validate"]);
    assert!(v.contains("E887"), "an IfAction without a condition: {v}");
    let bare = "---\ntype: ActionDef\nname: Bare\nvariables:\n  - {name: v, value: \"0\"}\nsubActions:\n  - {name: a, kind: Action}\n  - {name: b, kind: AssignmentAction, assign: \"v := a\"}\n---\n";
    let (_, v) = run(&model(&[("Bare", bare)]), &["validate"]);
    assert!(v.contains("W935"), "a bare sub-action name is not a value: {v}");
}
