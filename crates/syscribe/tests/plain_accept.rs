//! GH #513: in a composite, a plain `accept: X` (no `via:`) receives X from the part's only in port, so a machine
//! written to be used alone also works as a part. With several in ports the accept must name one.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const TX: &str = "---\ntype: StateDef\nname: TxM\nsubStates:\n  - name: Idle\n    isInitial: true\n    transitions:\n      - {internal: true, accept: Tick, effect: {send: Cmd, via: out}}\n---\n";

fn rx(accept: &str) -> String {
    format!("---\ntype: StateDef\nname: RxM\nsubStates:\n  - name: Waiting\n    isInitial: true\n    transitions:\n      - {{target: Done, accept: {accept}}}\n  - {{name: Done, isFinal: true}}\n---\n")
}

fn model(rx_accept: &str, rx_ports: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-plain-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    let files: Vec<(&str, String)> = vec![
        ("_index", "---\ntype: Package\nname: Root\n---\n".into()),
        ("TxM", TX.into()),
        ("RxM", rx(rx_accept)),
        ("TxP", "---\ntype: PartDef\nname: TxP\nexhibitsStates: [TxM]\nfeatures:\n  - {name: out, type: Port, direction: out}\n---\nTx.\n".into()),
        ("RxP", format!("---\ntype: PartDef\nname: RxP\nexhibitsStates: [RxM]\nfeatures:\n{rx_ports}---\nRx.\n")),
        ("Sys", "---\ntype: PartDef\nname: Sys\nfeatures:\n  - {name: tx, typedBy: TxP}\n  - {name: rx, typedBy: RxP}\nconnections:\n  - {from: tx.out, to: rx.inp, delay: \"5 ms\"}\n---\nSys.\n".into()),
        ("REQ-SYS-001", "---\ntype: Requirement\nid: REQ-SYS-001\nname: delivery\nstatus: approved\nreqDomain: software\n---\n\nThe receiver shall get the command.\n".into()),
        ("TC-SYS-001", "---\ntype: TestCase\nid: TC-SYS-001\nname: system\nstatus: active\ntestLevel: L3\nsubject: Sys\nverifies: [REQ-SYS-001]\n---\n\n```gherkin\nFeature: system\n\n  Scenario: the command arrives\n    When tx receives Tick\n    And 6 ms elapses\n    Then rx is in state Done\n```\n".into()),
    ];
    for (n, c) in files {
        std::fs::write(root.join(format!("{n}.md")), c).unwrap();
    }
    root
}

fn run(root: &PathBuf, args: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (o.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

const ONE_IN: &str = "  - {name: inp, type: Port, direction: in}\n";
const TWO_IN: &str = "  - {name: inp, type: Port, direction: in}\n  - {name: aux, type: Port, direction: in}\n";

#[test]
fn a_plain_accept_receives_a_port_message_when_the_part_has_one_in_port() {
    let root = model("Cmd", ONE_IN);
    let (c, t) = run(&root, &["simulate", "TC-SYS-001"]);
    assert_eq!(c, 0, "{t}");
    let (_, v) = run(&root, &["validate"]);
    assert!(!v.contains("W936"), "the plain accept is a handler for the connected port: {v}");
}

#[test]
fn an_explicit_via_accept_still_works_and_is_unchanged() {
    let (c, t) = run(&model("{payload: Cmd, via: inp}", ONE_IN), &["simulate", "TC-SYS-001"]);
    assert_eq!(c, 0, "{t}");
}

#[test]
fn with_several_in_ports_a_plain_accept_does_not_receive_and_w936_says_to_name_the_port() {
    let root = model("Cmd", TWO_IN);
    let (c, _) = run(&root, &["simulate", "TC-SYS-001"]);
    assert_ne!(c, 0, "ambiguous: a plain accept must not guess which port");
    let (_, v) = run(&root, &["validate"]);
    assert!(v.contains("W936") && v.contains("exactly one in port"), "{v}");
}

#[test]
fn a_part_with_two_in_ports_works_once_the_accept_names_its_port() {
    let (c, t) = run(&model("{payload: Cmd, via: inp}", TWO_IN), &["simulate", "TC-SYS-001"]);
    assert_eq!(c, 0, "{t}");
}

#[test]
fn the_same_machine_runs_alone_with_the_plain_event() {
    // the machine is not edited for the composite: a single-machine scenario injects the plain event
    let root = model("Cmd", ONE_IN);
    let (c, t) = run(&root, &["simulate", "RxM", "--do", "send Cmd"]);
    assert_eq!(c, 0, "{t}");
    assert!(t.contains("Done"), "{t}");
}
