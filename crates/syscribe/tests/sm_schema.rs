//! TC-TRS-SMSCHEMA-001 / REQ-TRS-SMSCHEMA-001 (GH #277): the executable state-machine schema and its checks.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model(machine: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-smschema-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(root.join("M.md"), format!("---\ntype: StateDef\nname: M\n{machine}---\n")).unwrap();
    root
}

fn validate(machine: &str) -> String {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(model(machine)).arg("validate").output().unwrap();
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

fn has(out: &str, code: &str) -> bool {
    out.lines().any(|l| l.contains(code))
}

/// Two states with the given transition lists (YAML, indented as list items under `transitions:`).
fn machine(extra: &str, a_transitions: &str, b_transitions: &str) -> String {
    format!(
        "{extra}subStates:\n  - name: A\n    isInitial: true\n    transitions:\n{a_transitions}  - name: B\n    transitions:\n{b_transitions}"
    )
}

const GOOD_A: &str = "      - {target: B, accept: {after: \"2 s\"}, guard: \"count < 3\", assign: \"count := count + 1\"}\n";
const GOOD_B: &str = "      - {target: A, accept: {when: \"count > 2\"}}\n      - {internal: true, accept: Pkg::Ping, assign: \"count := 0\"}\n";
const VARS: &str = "variables:\n  - {name: count, value: \"0\"}\ninitialEffect:\n  assign: \"count := 1\"\n";

#[test]
fn a_well_formed_executable_machine_validates_clean() {
    let o = validate(&machine(VARS, GOOD_A, GOOD_B));
    for c in ["E882", "E883", "E884", "E885", "W933", "W929", "W047"] {
        assert!(!has(&o, c), "{c} unexpected: {o}");
    }
}

#[test]
fn triggers_are_checked() {
    let two = "      - {target: B, accept: {after: \"1 s\", when: \"count > 1\"}}\n";
    assert!(has(&validate(&machine(VARS, two, "")), "E882"), "two trigger kinds");
    let bad_dur = "      - {target: B, accept: {after: \"2 kg\"}}\n";
    assert!(has(&validate(&machine(VARS, bad_dur, "")), "E882"), "after must be a duration");
    let unparse = "      - {target: B, accept: {when: \"count >> (\"}}\n";
    assert!(has(&validate(&machine(VARS, unparse, "")), "E882"), "unparseable when");
    let done = "      - {target: B, accept: {done: Nowhere}}\n";
    assert!(has(&validate(&machine(VARS, done, "")), "E882"), "done naming nothing");
    let key = "      - {target: B, accept: {sometimes: \"1\"}}\n";
    assert!(has(&validate(&machine(VARS, key, "")), "E882"), "unknown trigger key");
    let via = "      - {target: B, accept: {via: p}}\n";
    assert!(has(&validate(&machine(VARS, via, "")), "E882"), "via without payload");
    let done_ok = "      - {target: B, accept: {done: B}}\n";
    assert!(!has(&validate(&machine(VARS, done_ok, "")), "E882"), "done naming a state is fine");
}

#[test]
fn variables_and_assignments_are_checked() {
    let a = "      - {target: B, assign: \"count := 1\"}\n";
    assert!(has(&validate(&machine("variables:\n  - {name: \"1x\", value: \"0\"}\n", a, "")), "E883"), "bad name");
    assert!(has(&validate(&machine("variables:\n  - {name: count, value: \"0\"}\n  - {name: count, value: \"1\"}\n", a, "")), "E883"), "duplicate");
    assert!(has(&validate(&machine("variables:\n  - {name: count, value: \"1 +\"}\n", a, "")), "E883"), "unparseable value");
    assert!(has(&validate(&machine("variables:\n  - {name: count, value: \"other\"}\n", a, "")), "E883"), "value needing a name");
    let bad = "      - {target: B, assign: \"count = 1\"}\n";
    assert!(has(&validate(&machine(VARS, bad, "")), "E884"), "not a statement");
    let undeclared = "      - {target: B, assign: \"nope := 1\"}\n";
    assert!(has(&validate(&machine(VARS, undeclared, "")), "E884"), "undeclared variable");
    let two = "      - {target: B, assign: \"count := 1; count := count + 1\"}\n";
    assert!(!has(&validate(&machine(VARS, two, "")), "E884"), "several statements");
}

#[test]
fn internal_transitions_are_checked() {
    let with_target = "      - {internal: true, target: A, accept: Pkg::Ping}\n";
    assert!(has(&validate(&machine(VARS, "", with_target)), "E885"), "internal with target");
    let not_bool = "      - {internal: \"yes\", accept: Pkg::Ping}\n";
    assert!(has(&validate(&machine(VARS, "", not_bool)), "E885"), "internal not boolean");
    let internal = "      - {internal: true, accept: Pkg::Ping}\n";
    let o = validate(&machine(VARS, GOOD_A, internal));
    assert!(!has(&o, "W929"), "internal transition needs no target: {o}");
    // An internal transition is not an outgoing edge: B has no other way out, so it is still a trap state.
    assert!(has(&o, "W071"), "internal is not an outgoing edge: {o}");
}

#[test]
fn guards_of_executable_machines_are_checked_for_the_expression_language() {
    let opaque = "      - {target: B, accept: {after: \"1 s\"}, guard: \"the door is jammed\"}\n";
    assert!(has(&validate(&machine(VARS, opaque, "")), "W933"), "opaque guard in an executable machine");
    let unknown = "      - {target: B, accept: {after: \"1 s\"}, guard: \"missing > 1\"}\n";
    assert!(has(&validate(&machine(VARS, unknown, "")), "W933"), "name that is nothing");
    let ok = "      - {target: B, accept: {after: \"1 s\"}, guard: \"count > 1\"}\n";
    assert!(!has(&validate(&machine(VARS, ok, "")), "W933"));
}

#[test]
fn existing_models_see_nothing_new() {
    let plain = "      - {target: B, accept: Pkg::Go, guard: \"the door is jammed\"}\n";
    let back = "      - {target: A, accept: Pkg::Back}\n";
    let o = validate(&machine("", plain, back));
    for c in ["E882", "E883", "E884", "E885", "W933"] {
        assert!(!has(&o, c), "{c} on a non-executable machine: {o}");
    }
}

#[test]
fn state_action_assignments_and_effect_maps_are_checked() {
    let v = "variables:\n  - {name: count, value: \"0\"}\n";
    let y = format!("{v}subStates:\n  - name: A\n    isInitial: true\n    entryAction: {{assign: \"zz := 1\"}}\n    transitions:\n      - {{target: B, accept: Pkg::Go}}\n  - name: B\n    transitions:\n      - {{target: A, accept: Pkg::Back, effect: {{assign: \"qq := 1\"}}}}\n");
    let o = validate(&y);
    assert!(o.matches("E884").count() >= 2, "entryAction and effect assigns: {o}");
}

#[test]
fn done_may_name_a_dotted_path_and_an_empty_accept_is_rejected() {
    let nested = "subStates:\n  - name: P\n    isInitial: true\n    transitions: [{target: Z, accept: {done: P.R}}]\n    subStates:\n      - {name: R, isInitial: true, subStates: [{name: f, isInitial: true}]}\n  - {name: Z}\n";
    assert!(!has(&validate(nested), "E882"), "dotted done path");
    let empty = "subStates:\n  - name: A\n    isInitial: true\n    transitions: [{target: B, accept: {}}]\n  - {name: B}\n";
    assert!(has(&validate(empty), "E882"), "accept with nothing");
}

#[test]
fn variable_units_are_checked() {
    assert!(has(&validate("variables:\n  - {name: v, value: \"5\", unit: \"furlongs\"}\nsubStates:\n  - {name: A, isInitial: true}\n"), "E883"), "unknown unit");
    assert!(has(&validate("variables:\n  - {name: v, value: \"5 s\", unit: \"kg\"}\nsubStates:\n  - {name: A, isInitial: true}\n"), "E883"), "unit on a value that already has one");
}
