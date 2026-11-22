//! GH #512 probe: in a composite, a port message re-enters a state that has two `after:` timers.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model(delay: &str, scenario: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-timers-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    let files: Vec<(&str, String)> = vec![
        ("_index", "---\ntype: Package\nname: Root\n---\n".into()),
        ("TxM", "---\ntype: StateDef\nname: TxM\nsubStates:\n  - name: Idle\n    isInitial: true\n    transitions:\n      - {internal: true, accept: Tick, effect: {send: Kick, via: out}}\n---\n".into()),
        ("RxM", "---\ntype: StateDef\nname: RxM\nsubStates:\n  - name: A\n    isInitial: true\n    transitions:\n      - {target: B, accept: {after: \"2 s\"}}\n      - {target: C, accept: {after: \"3 s\"}}\n      - {target: A, accept: {payload: Kick, via: inp}}\n  - name: B\n  - name: C\n---\n".into()),
        ("TxP", "---\ntype: PartDef\nname: TxP\nexhibitsStates: [TxM]\nfeatures:\n  - {name: out, type: Port, direction: out}\n---\nTx.\n".into()),
        ("RxP", "---\ntype: PartDef\nname: RxP\nexhibitsStates: [RxM]\nfeatures:\n  - {name: inp, type: Port, direction: in}\n---\nRx.\n".into()),
        ("Sys", format!("---\ntype: PartDef\nname: Sys\nfeatures:\n  - {{name: tx, typedBy: TxP}}\n  - {{name: rx, typedBy: RxP}}\nconnections:\n  - {{from: tx.out, to: rx.inp, delay: \"{delay}\"}}\n---\nSys.\n")),
        ("REQ-SYS-001", "---\ntype: Requirement\nid: REQ-SYS-001\nname: r\nstatus: approved\nreqDomain: software\n---\n\nr\n".into()),
        ("TC-SYS-001", format!("---\ntype: TestCase\nid: TC-SYS-001\nname: system\nstatus: active\ntestLevel: L3\nsubject: Sys\nverifies: [REQ-SYS-001]\n---\n\n```gherkin\nFeature: system\n\n{scenario}```\n")),
    ];
    for (n, c) in files {
        std::fs::write(root.join(format!("{n}.md")), c).unwrap();
    }
    root
}

fn simulate(root: &PathBuf) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(["simulate", "TC-SYS-001"]).output().unwrap();
    (o.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

#[test]
fn a_message_that_re_enters_a_state_restarts_both_of_its_timers() {
    // the kick lands at 1 s; the timers restart: due at 3 s and 4 s
    let root = model("1 s", "  Scenario: restart\n    When tx receives Tick\n    And 2.9 s elapses\n    Then rx is in state A\n    When 0.2 s elapses\n    Then rx is in state B\n    When 5 s elapses\n    Then rx is in state B\n");
    let (c, t) = simulate(&root);
    assert_eq!(c, 0, "{t}");
}

#[test]
fn a_message_landing_before_at_or_after_a_timer_is_due_never_lets_the_other_timer_survive() {
    // delays 1.5 s (before the 2 s timer), 2 s (the same instant) and 2.5 s (after it, once A has gone to B)
    for delay in ["1.5 s", "2 s", "2.5 s"] {
        let ends_in_b = simulate(&model(delay, "  Scenario: s\n    When tx receives Tick\n    And 10 s elapses\n    Then rx is in state B\n"));
        assert_eq!(ends_in_b.0, 0, "delay {delay}: rx must end in B: {}", ends_in_b.1);
        let ends_in_c = simulate(&model(delay, "  Scenario: s\n    When tx receives Tick\n    And 10 s elapses\n    Then rx is in state C\n"));
        assert_ne!(ends_in_c.0, 0, "delay {delay}: the 3 s timer of the old A must not fire");
    }
}
