//! GH #511 / ADR-SYS-AFTER-001: `after:` takes an expression over the machine's variables, read when the timer is armed.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model(machine: &str, scenarios: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-afterexpr-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(root.join("M.md"), format!("---\ntype: StateDef\nname: M\n{machine}---\n")).unwrap();
    std::fs::write(
        root.join("TC-T-001.md"),
        format!("---\ntype: TestCase\nid: TC-T-001\nname: t\nstatus: active\ntestLevel: L2\nsubject: M\n---\n\n```gherkin\nFeature: t\n\n{scenarios}```\n"),
    )
    .unwrap();
    root
}

fn run(root: &PathBuf, args: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (o.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

fn machine(var: &str, after: &str) -> String {
    format!("variables:\n  - {var}\nsubStates:\n  - name: A\n    isInitial: true\n    transitions: [{{target: B, accept: {{after: \"{after}\"}}}}]\n  - name: B\n")
}

const TIMED: &str = "  Scenario: fires at the deadline
    Given M is in state A
    When 1.9 s elapses
    Then M is in state A
    When 0.2 s elapses
    Then M is in state B
";

#[test]
fn a_variable_deadline_runs_in_simulate_and_validates_clean() {
    let root = model(&machine("{name: deadline, value: \"2 s\"}", "deadline"), TIMED);
    let (c, t) = run(&root, &["simulate", "TC-T-001"]);
    assert_eq!(c, 0, "{t}");
    let (_, v) = run(&root, &["validate"]);
    assert!(!v.contains("E882") && !v.contains("E935") && !v.contains("W933"), "{v}");
}

#[test]
fn a_dimensionless_variable_is_not_a_duration_e935() {
    let (_, v) = run(&model(&machine("{name: deadline, value: \"2\"}", "deadline"), TIMED), &["validate"]);
    assert!(v.contains("E935") && v.contains("not a duration"), "{v}");
}

#[test]
fn a_name_that_is_not_a_variable_is_w933_and_a_bad_literal_is_still_e882() {
    let (_, v) = run(&model(&machine("{name: deadline, value: \"2 s\"}", "missing"), TIMED), &["validate"]);
    assert!(v.contains("W933") && v.contains("missing"), "{v}");
    let (_, v) = run(&model(&machine("{name: deadline, value: \"2 s\"}", "0 s - 1 s"), TIMED), &["validate"]);
    assert!(v.contains("E882"), "{v}");
}

#[test]
fn mutate_scales_an_expression_timer_and_the_timing_scenario_kills_both() {
    let root = model(&machine("{name: deadline, value: \"2 s\"}", "deadline"), TIMED);
    let (c, t) = run(&root, &["mutate", "M", "--json"]);
    assert_eq!(c, 0, "{t}");
    let j: serde_json::Value = serde_json::from_str(&t).unwrap_or_else(|e| panic!("{e}: {t}"));
    let scaled: Vec<&serde_json::Value> = j["mutants"].as_array().unwrap().iter().filter(|m| m["operator"] == "scale-after").collect();
    assert_eq!(scaled.len(), 2, "{t}");
    assert!(scaled.iter().all(|m| m["fate"] == "killed" && m["description"].as_str().unwrap().contains("deadline")), "{t}");
}

#[test]
fn coverage_labels_the_timer_with_its_expression() {
    let root = model(&machine("{name: deadline, value: \"2 s\"}", "deadline"), TIMED);
    let (_, t) = run(&root, &["simulate", "TC-T-001", "--coverage", "--json"]);
    assert!(t.contains("after deadline"), "{t}");
}

#[test]
fn scxml_export_carries_the_expression_as_delayexpr() {
    let root = model(&machine("{name: deadline, value: \"2 s\"}", "deadline"), TIMED);
    let (c, t) = run(&root, &["export-scxml", "M"]);
    assert_eq!(c, 0, "{t}");
    assert!(t.contains("delayexpr=\"((deadline) * 1000) + &apos;ms&apos;\"") || t.contains("delayexpr=\"((deadline) * 1000) + 'ms'\""), "{t}");
    assert!(!t.contains("no SCXML form"), "{t}");
}
