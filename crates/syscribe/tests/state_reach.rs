//! GH #515: W070 (dead state) and W071 (trap state) follow transitions declared on ancestors and inside descendants.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn validate(machine: &str) -> String {
    static N: AtomicU64 = AtomicU64::new(0);
    let root: PathBuf = std::env::temp_dir().join(format!("syscribe-reach-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(root.join("M.md"), format!("---\ntype: StateDef\nname: M\n{machine}---\n")).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&root).arg("validate").output().unwrap();
    String::from_utf8_lossy(&o.stdout).to_string()
}

#[test]
fn a_state_entered_only_by_a_transition_declared_on_its_parent_is_not_dead() {
    let t = validate("subStates:\n  - name: Live\n    isInitial: true\n    transitions:\n      - {target: Parked, accept: Park}\n    subStates:\n      - {name: Running, isInitial: true}\n      - {name: Parked, transitions: [{target: Running, accept: Go}]}\n");
    assert!(!t.contains("W070") && !t.contains("W071"), "{t}");
}

#[test]
fn a_state_entered_only_from_a_nested_state_is_not_dead() {
    let t = validate("subStates:\n  - name: Live\n    isInitial: true\n    subStates:\n      - {name: Running, isInitial: true, transitions: [{target: Off, accept: Stop}]}\n  - {name: Off, transitions: [{target: Live, accept: Start}]}\n");
    assert!(!t.contains("W070") && !t.contains("W071"), "{t}");
}

#[test]
fn a_leaf_is_not_a_trap_when_its_parent_has_the_outgoing_transition() {
    let t = validate("subStates:\n  - name: Live\n    isInitial: true\n    transitions:\n      - {target: Off, accept: Stop}\n    subStates:\n      - {name: Running, isInitial: true}\n  - {name: Off, isFinal: true}\n");
    assert!(!t.contains("W071"), "Running leaves with its parent: {t}");
}

#[test]
fn a_dotted_target_path_counts_as_entering_the_state() {
    let t = validate("subStates:\n  - name: Live\n    isInitial: true\n    subStates:\n      - {name: Running, isInitial: true, transitions: [{target: Live.Parked, accept: Park}]}\n      - {name: Parked, transitions: [{target: Live.Running, accept: Go}]}\n");
    assert!(!t.contains("W070"), "{t}");
}

#[test]
fn a_state_nothing_enters_is_still_dead_and_one_nothing_leaves_is_still_a_trap() {
    let t = validate("subStates:\n  - name: A\n    isInitial: true\n    transitions: [{target: Sink, accept: X}]\n  - name: Sink\n  - name: Orphan\n    transitions: [{target: A, accept: Y}]\n");
    assert!(t.contains("W070") && t.contains("'Orphan'"), "{t}");
    assert!(t.contains("W071") && t.contains("'Sink'"), "{t}");
}
