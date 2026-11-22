//! GH #516 / #509: an `ActionDef` made only of `SendAction`s that opts in (`variables: []`) sends its messages in
//! `simulate` as a transition effect, the composite checks (W936) see those sends, and one that has not opted in is
//! reported by W937 instead of being silent.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const BEEP_ONLY: &str = "---\ntype: ActionDef\nname: Beep\nvariables: []\nsubActions:\n  - {name: tx0, kind: SendAction, payload: Cmd, via: out}\n---\n";
/// No `variables:` and nothing assigned: not executable, so a machine that runs it records the reference only.
const BEEP_UNDECLARED: &str = "---\ntype: ActionDef\nname: Beep\nsubActions:\n  - {name: tx0, kind: SendAction, payload: Cmd, via: out}\n---\n";
const BEEP_COUNTED: &str = "---\ntype: ActionDef\nname: Beep\nvariables:\n  - {name: n, value: \"0\"}\nsubActions:\n  - {name: inc, kind: AssignmentAction, assign: \"n := n + 1\"}\n  - {name: tx0, kind: SendAction, payload: Cmd, via: out}\n---\n";

fn files(beep: &str) -> Vec<(&'static str, String)> {
    vec![
        ("Beep", beep.to_string()),
        ("TxM", "---\ntype: StateDef\nname: TxM\nsubStates:\n  - name: Idle\n    isInitial: true\n    transitions:\n      - {internal: true, accept: Tick, effect: Beep}\n---\n".into()),
        ("RxM", "---\ntype: StateDef\nname: RxM\nsubStates:\n  - name: Waiting\n    isInitial: true\n    transitions:\n      - {target: Done, accept: {payload: Cmd, via: inp}}\n  - {name: Done, isFinal: true}\n---\n".into()),
        ("TxP", "---\ntype: PartDef\nname: TxP\nexhibitsStates: [TxM]\nfeatures:\n  - {name: out, type: Port, direction: out}\n---\nTransmitter.\n".into()),
        ("RxP", "---\ntype: PartDef\nname: RxP\nexhibitsStates: [RxM]\nfeatures:\n  - {name: inp, type: Port, direction: in}\n---\nReceiver.\n".into()),
        ("Sys", "---\ntype: PartDef\nname: Sys\nfeatures:\n  - {name: tx, typedBy: TxP}\n  - {name: rx, typedBy: RxP}\nconnections:\n  - {from: tx.out, to: rx.inp, delay: \"5 ms\"}\n---\nThe system.\n".into()),
        ("REQ-SYS-001", "---\ntype: Requirement\nid: REQ-SYS-001\nname: delivery\nstatus: approved\nreqDomain: software\n---\n\nThe receiver shall get the command.\n".into()),
        ("TC-SYS-001", "---\ntype: TestCase\nid: TC-SYS-001\nname: system\nstatus: active\ntestLevel: L3\nsubject: Sys\nverifies: [REQ-SYS-001]\n---\n\n```gherkin\nFeature: system\n\n  Scenario: the effect's send arrives\n    When tx receives Tick\n    Then tx sent Cmd via out\n    When 6 ms elapses\n    Then rx is in state Done\n```\n".into()),
    ]
}

fn model(beep: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-effsend-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    for (n, c) in files(beep) {
        std::fs::write(root.join(format!("{n}.md")), c).unwrap();
    }
    root
}

fn run(root: &PathBuf, args: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (o.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

#[test]
fn an_action_of_sends_alone_sends_when_used_as_an_effect() {
    let (c, t) = run(&model(BEEP_ONLY), &["simulate", "TC-SYS-001"]);
    assert_eq!(c, 0, "{t}");
    assert!(t.contains("1/1 scenario(s) pass"), "{t}");
}

#[test]
fn an_action_with_an_assignment_before_the_send_still_works() {
    let (c, t) = run(&model(BEEP_COUNTED), &["simulate", "TC-SYS-001"]);
    assert_eq!(c, 0, "{t}");
}

#[test]
fn w936_follows_sends_inside_effect_action_defs() {
    for beep in [BEEP_ONLY, BEEP_COUNTED] {
        let (_, t) = run(&model(beep), &["validate"]);
        assert!(!t.contains("W936"), "a send inside an effect ActionDef is a real sender: {t}");
    }
}

#[test]
fn w936_still_fires_when_nothing_sends() {
    let silent = "---\ntype: ActionDef\nname: Beep\nvariables:\n  - {name: n, value: \"0\"}\nsubActions:\n  - {name: inc, kind: AssignmentAction, assign: \"n := n + 1\"}\n---\n";
    let (_, t) = run(&model(silent), &["validate"]);
    assert!(t.contains("W936"), "{t}");
}

#[test]
fn an_undeclared_sends_only_action_is_explained_by_w937_and_sends_nothing() {
    let root = model(BEEP_UNDECLARED);
    let (_, t) = run(&root, &["validate"]);
    assert!(t.contains("W937") && t.contains("variables: []"), "{t}");
    let (c, _) = run(&root, &["simulate", "TC-SYS-001"]);
    assert_ne!(c, 0, "the documented opt-in rule is unchanged: nothing is sent");
}

#[test]
fn w937_is_silent_once_the_action_opts_in() {
    for beep in [BEEP_ONLY, BEEP_COUNTED] {
        let (_, t) = run(&model(beep), &["validate"]);
        assert!(!t.contains("W937"), "{t}");
    }
}
