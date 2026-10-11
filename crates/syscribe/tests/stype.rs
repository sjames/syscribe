//! TC-TRS-STYPE-001 / REQ-TRS-STYPE-001: static type and dimension checking of executable expressions (E935).

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model(machine: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-stype-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(root.join("M.md"), format!("---\ntype: StateDef\nname: M\n{machine}---\nA machine.\n")).unwrap();
    root
}

fn run(root: &PathBuf, args: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (o.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

const VARS: &str = "variables:
  - {name: height, value: \"1\", unit: \"SI::m\"}
  - {name: count, value: \"0\"}
  - {name: armed, value: \"false\"}
  - {name: speed, value: \"2 m / 1 s\"}
";

/// One machine with the given guard and assign on a transition A → B.
fn with(guard: &str, assign: &str) -> String {
    let g = if guard.is_empty() { String::new() } else { format!(", guard: \"{guard}\"") };
    let a = if assign.is_empty() { String::new() } else { format!(", assign: \"{assign}\"") };
    format!("{VARS}subStates:\n  - name: A\n    isInitial: true\n    transitions:\n      - {{target: B, accept: Go{g}{a}}}\n  - name: B\n    transitions:\n      - {{target: A, accept: Back}}\n")
}

fn e935(machine: &str) -> (bool, String) {
    let (_, t) = run(&model(machine), &["validate"]);
    (t.contains("E935"), t)
}

#[test]
fn a_well_typed_machine_is_clean() {
    let (bad, t) = e935(&with("height > 0.5 m and count < 3 and not armed", "count := count + 1; height := height * 2"));
    assert!(!bad, "{t}");
}

#[test]
fn dimension_and_kind_errors_are_found_without_running() {
    for (guard, assign, what) in [
        ("height < 5", "", "metres against a plain number"),
        ("count + 1", "", "a guard that is not boolean"),
        ("", "height := count", "assigning a number to a length"),
        ("", "armed := 1", "assigning a number to a boolean"),
        ("count and armed", "", "and over a number"),
        ("height + count > 1 m", "", "adding a length to a number"),
        ("armed < true", "", "ordering booleans"),
        ("height == armed", "", "comparing a number with a boolean"),
    ] {
        let (bad, t) = e935(&with(guard, assign));
        assert!(bad, "{what} should be E935: {t}");
    }
}

#[test]
fn functions_follow_the_evaluator() {
    for (guard, what) in [
        ("min(height, count) > 0 m", "min over a length and a number"),
        ("sqrt(height) > 0 m", "sqrt of an odd dimension"),
        ("pow(height, 1.5) > 0 m", "a non-integer exponent"),
        ("pow(height, height) > 0 m", "a dimensional exponent"),
    ] {
        let (bad, t) = e935(&with(guard, ""));
        assert!(bad, "{what} should be E935: {t}");
    }
    let (bad, t) = e935(&with("sqrt(height * height) > 0.5 m and pow(height, 2) > 0 m * 1 m and speed * 2 s > 1 m", ""));
    assert!(!bad, "even dimensions, an integer exponent and derived units are fine: {t}");
}

#[test]
fn names_that_are_not_variables_are_never_a_type_error() {
    let m = format!("{VARS}subStates:\n  - name: A\n    isInitial: true\n    transitions:\n      - {{target: B, accept: {{payload: Msg, as: m}}, guard: \"m.level > 5 and m.flag\", assign: \"count := m.n + 1\"}}\n  - name: B\n    transitions:\n      - {{target: A, accept: Back}}\n");
    let (bad, t) = e935(&m);
    assert!(!bad, "{t}");
}

#[test]
fn send_payloads_and_state_actions_are_checked_and_plain_machines_are_not() {
    let m = format!("{VARS}subStates:\n  - name: A\n    isInitial: true\n    entryAction: {{assign: \"count := height\"}}\n    transitions:\n      - {{target: A, accept: Go, effect: {{send: Out, via: p, payload: {{x: \"height + count\"}}}}}}\n");
    let (_, t) = run(&model(&m), &["validate"]);
    assert!(t.matches("E935").count() >= 2, "entry assignment and payload: {t}");
    // a machine that is not executable (prose guards, no variables) is left alone
    let plain = "subStates:\n  - name: A\n    isInitial: true\n    transitions:\n      - {target: B, accept: Go, guard: \"the door is jammed\"}\n  - name: B\n    transitions:\n      - {target: A, accept: Back}\n";
    let (bad, t) = e935(plain);
    assert!(!bad, "{t}");
}

#[test]
fn compile_refuses_what_validate_reports() {
    let root = model(&with("height < 5", ""));
    let (c, t) = run(&root, &["compile", "M"]);
    assert_ne!(c, 0, "{t}");
    assert!(t.contains("E935"), "{t}");
}

#[test]
fn another_lowering_diagnostic_does_not_hide_a_type_error() {
    // the first guard is prose (W933); the second is a certain dimension error elsewhere in the same machine
    let m = format!("{VARS}subStates:\n  - name: A\n    isInitial: true\n    transitions:\n      - {{target: B, accept: Go, guard: \"the door is jammed\"}}\n      - {{target: B, accept: Go2, guard: \"height < 5\"}}\n  - name: B\n    transitions:\n      - {{target: A, accept: Back}}\n");
    let (bad, t) = e935(&m);
    assert!(bad, "{t}");
    assert!(t.contains("W933"), "{t}");
}

#[test]
fn a_negative_literal_exponent_is_checked_like_a_positive_one() {
    let (bad, t) = e935(&with("pow(height, -2000) > 0 m", ""));
    assert!(bad, "{t}");
    let (bad, t) = e935(&with("pow(height, -1) * height > 0.5", ""));
    assert!(!bad, "{t}");
}
