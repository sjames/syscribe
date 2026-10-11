//! TC-TRS-SYSCOMP-001 / REQ-TRS-SYSCOMP-001 (GH #297): the `send` effect of a state machine.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::expr::{format_value, parse_value};
use syscribe_model::ir::{lower, Machine};
use syscribe_model::sm_run::{Options, Run};
use syscribe_model::walker::walk_model;

fn machine(yaml: &str) -> Machine {
    static N: AtomicU64 = AtomicU64::new(0);
    let root: PathBuf = std::env::temp_dir().join(format!("syscribe-smsend-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(root.join("M.md"), format!("---\ntype: StateDef\nname: M\n{yaml}---\n")).unwrap();
    lower(&walk_model(&root).unwrap(), "M").unwrap_or_else(|d| panic!("lowering failed: {d:?}"))
}

const SENDER: &str = "variables:
  - {name: level, value: \"1\"}
initialEffect: {send: Hello, via: out}
subStates:
  - name: Idle
    isInitial: true
    transitions:
      - target: Busy
        accept: Go
        effect: {send: Work, via: out, payload: {amount: \"level + 1\", flag: \"true\"}, assign: \"level := level + 1\"}
  - name: Busy
    entryAction: {send: Entered}
";

#[test]
fn a_send_effect_lands_in_the_outbox_with_its_payload_evaluated_at_send_time() {
    let m = machine(SENDER);
    let mut r = Run::new(&m, Options::default());
    let first = r.take_outbox();
    assert_eq!(first.len(), 1, "the initial effect sends once");
    assert_eq!((first[0].event.as_str(), first[0].via.as_deref()), ("Hello", Some("out")));
    assert!(r.take_outbox().is_empty(), "taking empties the outbox");
    r.send("Go", &[]);
    let sent = r.take_outbox();
    let names: Vec<&str> = sent.iter().map(|s| s.event.as_str()).collect();
    assert_eq!(names, ["Work", "Entered"], "the transition effect precedes the entry action of the target");
    let amount = sent[0].payload.iter().find(|(k, _)| k == "amount").map(|(_, v)| format_value(v)).unwrap();
    assert_eq!(amount, format_value(&parse_value("3").unwrap()), "within one effect the assignment runs first, so the payload sees level = 2");
    assert_eq!(sent[1].via, None);
    assert!(r.trace().iter().any(|s| s.actions.iter().any(|a| a.starts_with("send Work via out"))), "traced");
}

#[test]
fn machines_without_send_keep_their_ir_and_hash() {
    let plain = machine("variables:\n  - {name: n, value: \"0\"}\nsubStates:\n  - {name: A, isInitial: true, transitions: [{target: B, accept: Go, assign: \"n := 1\"}]}\n  - {name: B}\n");
    assert!(!plain.canonical_json().contains("\"send\""));
    let with = machine(SENDER);
    assert!(with.canonical_json().contains("\"kind\":\"send\""));
    assert_ne!(plain.hash, with.hash);
}
