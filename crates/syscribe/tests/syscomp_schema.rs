//! TC-TRS-SYSCOMP-001 / REQ-TRS-SYSCOMP-001 (GH #297): composite parts — resolution and static checks.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const SENDER: &str = "---
type: StateDef
name: SenderMachine
variables:
  - {name: level, value: \"0\"}
  - {name: dist, value: \"0\", unit: \"SI::m\"}
subStates:
  - name: Idle
    isInitial: true
    transitions:
      - {internal: true, accept: Tick, effect: {send: Cmd, via: out, payload: {n: \"level\"}}}
---
";
const RECEIVER: &str = "---
type: StateDef
name: ReceiverMachine
variables:
  - {name: got, value: \"0\"}
  - {name: dist, value: \"0\", unit: \"SI::m\"}
subStates:
  - name: Waiting
    isInitial: true
    transitions:
      - {target: Done, accept: {payload: Cmd, via: inp, as: m}}
  - name: Done
---
";
const SENDER_DEF: &str = "---
type: PartDef
name: SenderPart
exhibitsStates: [SenderMachine]
features:
  - {name: out, type: Port, direction: out}
---
";
const RECEIVER_DEF: &str = "---
type: PartDef
name: ReceiverPart
exhibitsStates: [ReceiverMachine]
features:
  - {name: inp, type: Port, direction: in}
---
";
const SYSTEM: &str = "---
type: PartDef
name: System
features:
  - {name: tx, typedBy: SenderPart}
  - {name: rx, typedBy: ReceiverPart}
connections:
  - {from: tx.out, to: rx.inp}
bindingConnections:
  - {from: tx.dist, to: rx.dist}
---
";

fn model(files: &[(&str, &str)]) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-syscomp-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    let mut all: Vec<(&str, &str)> = vec![("SenderMachine", SENDER), ("ReceiverMachine", RECEIVER), ("SenderPart", SENDER_DEF), ("ReceiverPart", RECEIVER_DEF), ("System", SYSTEM)];
    for (n, c) in files {
        all.retain(|(m, _)| m != n);
        all.push((n, c));
    }
    for (n, c) in all {
        std::fs::write(root.join(format!("{n}.md")), c).unwrap();
    }
    root
}

fn validate(files: &[(&str, &str)]) -> String {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(model(files)).arg("validate").output().unwrap();
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

fn has(out: &str, code: &str) -> bool {
    out.lines().any(|l| l.contains(code))
}

const CODES: [&str; 6] = ["E930", "E931", "E932", "E933", "E934", "W936"];

fn clean(out: &str) {
    for c in CODES {
        assert!(!has(out, c), "{c} unexpected: {out}");
    }
}

#[test]
fn a_well_formed_composite_validates_clean() {
    let o = validate(&[]);
    clean(&o);
    assert!(!o.contains("Errors ("), "the fixture itself has no errors: {o}");
}

#[test]
fn a_connection_must_join_real_ports_in_the_right_direction() {
    let swapped = SYSTEM.replace("{from: tx.out, to: rx.inp}", "{from: rx.inp, to: tx.out}");
    assert!(has(&validate(&[("System", &swapped)]), "E931"), "direction");
    let nowhere = SYSTEM.replace("rx.inp", "rx.nowhere");
    assert!(has(&validate(&[("System", &nowhere)]), "E931"), "unknown port");
    let delay = SYSTEM.replace("{from: tx.out, to: rx.inp}", "{from: tx.out, to: rx.inp, delay: \"soon\"}");
    assert!(has(&validate(&[("System", &delay)]), "E934"), "bad delay");
    let ok = SYSTEM.replace("{from: tx.out, to: rx.inp}", "{from: tx.out, to: rx.inp, delay: \"5 ms\"}");
    clean(&validate(&[("System", &ok)]));
}

#[test]
fn a_machine_may_only_use_ports_its_part_has() {
    let bad_send = SENDER.replace("via: out", "via: elsewhere");
    assert!(has(&validate(&[("SenderMachine", &bad_send)]), "E932"), "send via an unknown port");
    let bad_accept = RECEIVER.replace("via: inp", "via: out");
    assert!(has(&validate(&[("ReceiverMachine", &bad_accept)]), "E932"), "accept via an unknown port");
}

#[test]
fn a_part_has_exactly_one_behaviour() {
    let two = SENDER_DEF.replace("exhibitsStates: [SenderMachine]", "exhibitsStates: [SenderMachine, ReceiverMachine]");
    assert!(has(&validate(&[("SenderPart", &two)]), "E930"), "two machines");
    let dangling = SENDER_DEF.replace("[SenderMachine]", "[Nowhere]");
    assert!(has(&validate(&[("SenderPart", &dangling)]), "E930"), "unresolved machine");
}

#[test]
fn bindings_must_join_variables_of_one_type_and_dimension() {
    let unknown = SYSTEM.replace("rx.dist", "rx.ghost");
    assert!(has(&validate(&[("System", &unknown)]), "E933"), "unknown variable");
    let dim = SYSTEM.replace("{from: tx.dist, to: rx.dist}", "{from: tx.dist, to: rx.got}");
    assert!(has(&validate(&[("System", &dim)]), "E933"), "metres to a plain number");
}

#[test]
fn traffic_nobody_handles_is_a_warning() {
    let other = SENDER.replace("send: Cmd", "send: Other");
    let o = validate(&[("SenderMachine", &other)]);
    assert!(has(&o, "W936"), "sent but not accepted, accepted but never sent: {o}");
}

#[test]
fn a_part_composition_without_executable_machines_sees_nothing_new() {
    let plain_sender = "---\ntype: StateDef\nname: SenderMachine\nsubStates:\n  - {name: A, isInitial: true}\n---\n";
    let plain_receiver = "---\ntype: StateDef\nname: ReceiverMachine\nsubStates:\n  - {name: A, isInitial: true}\n---\n";
    let bad = SYSTEM.replace("rx.inp", "rx.nowhere");
    clean(&validate(&[("SenderMachine", plain_sender), ("ReceiverMachine", plain_receiver), ("System", &bad)]));
}

#[test]
fn a_typo_in_a_part_name_is_reported_not_dropped() {
    assert!(has(&validate(&[("System", &SYSTEM.replace("{from: tx.out, to: rx.inp}", "{from: txx.out, to: rx.inp}"))]), "E931"), "connection");
    assert!(has(&validate(&[("System", &SYSTEM.replace("{from: tx.dist, to: rx.dist}", "{from: txx.dist, to: rx.dist}"))]), "E933"), "binding");
}

#[test]
fn a_relay_without_variables_is_still_an_executable_machine() {
    let relay = "---\ntype: StateDef\nname: SenderMachine\nsubStates:\n  - name: Idle\n    isInitial: true\n    transitions:\n      - {internal: true, accept: {payload: Tick, via: out}, effect: {send: Cmd, via: nowhere}}\n---\n";
    assert!(has(&validate(&[("SenderMachine", relay)]), "E932"), "the port checks apply to it");
}

#[test]
fn send_checks_cover_entry_and_exit_actions() {
    let bad = SENDER.replace("subStates:\n  - name: Idle\n    isInitial: true\n", "subStates:\n  - name: Idle\n    isInitial: true\n    entryAction: {send: Oops, payload: [1]}\n");
    assert!(has(&validate(&[("SenderMachine", &bad)]), "E884"));
}

#[test]
fn fan_in_and_environment_events_are_not_w936_false_positives() {
    let rx_extra = RECEIVER.replace("  - name: Done\n", "  - name: Done\n    transitions:\n      - {target: Waiting, accept: Reset}\n");
    let o = validate(&[("ReceiverMachine", &rx_extra)]);
    assert!(!has(&o, "W936"), "an event injected from the environment is fine: {o}");
}

#[test]
fn a_binding_needs_a_source_per_sink_and_a_part_with_behaviour() {
    let idle = "---\ntype: PartDef\nname: IdleP\n---\nNo behaviour.\n";
    let sys = SYSTEM.replace("  - {name: rx, typedBy: ReceiverPart}\n", "  - {name: rx, typedBy: ReceiverPart}\n  - {name: idle, typedBy: IdleP}\n").replace("{from: tx.dist, to: rx.dist}", "{from: tx.dist, to: idle.x}");
    assert!(has(&validate(&[("System", &sys), ("IdleP", idle)]), "E933"), "inert part");
    let twice = SYSTEM.replace("{from: tx.dist, to: rx.dist}", "{from: tx.dist, to: rx.dist}\n  - {from: tx.level, to: rx.dist}");
    assert!(has(&validate(&[("System", &twice)]), "E933"), "two sources");
}

#[test]
fn ports_may_be_separate_child_elements() {
    // the ports of the two parts as `Def/port.md` files instead of inline features
    let tx = "---\ntype: PartDef\nname: SenderPart\nexhibitsStates: [SenderMachine]\n---\nTransmitter.\n";
    let rx = "---\ntype: PartDef\nname: ReceiverPart\nexhibitsStates: [ReceiverMachine]\n---\nReceiver.\n";
    let root = model(&[("SenderPart", tx), ("ReceiverPart", rx)]);
    std::fs::create_dir_all(root.join("SenderPart")).unwrap();
    std::fs::create_dir_all(root.join("ReceiverPart")).unwrap();
    std::fs::write(root.join("SenderPart/out.md"), "---\ntype: Port\nname: \"Out port of the transmitter\"\ndirection: out\n---\nOut.\n").unwrap();
    std::fs::write(root.join("ReceiverPart/inp.md"), "---\ntype: Port\nname: inp\ndirection: in\n---\nIn.\n").unwrap();
    let run = |root: &PathBuf| {
        let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).arg("validate").output().unwrap();
        format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
    };
    let o = run(&root);
    clean(&o);
    assert!(!o.contains("Errors ("), "child ports are found: {o}");
    // a direction clash is detected on child ports too
    std::fs::write(root.join("SenderPart/out.md"), "---\ntype: Port\nname: out\ndirection: in\n---\nIn.\n").unwrap();
    assert!(has(&run(&root), "E931"), "an in port cannot send");
}

#[test]
fn a_composite_inherits_the_wiring_of_its_supertype() {
    let derived = "---\ntype: PartDef\nname: Derived\nsupertype: System\n---\nA variant of the system.\n";
    let o = validate(&[("Derived", derived)]);
    clean(&o);
    // an error in the inherited wiring is found on the variant as well
    let bad_base = SYSTEM.replace("rx.inp", "rx.nowhere");
    let o = validate(&[("System", &bad_base), ("Derived", derived)]);
    let rows: Vec<&str> = o.lines().filter(|l| l.contains("E931")).collect();
    assert_eq!(rows.len(), 1, "reported once, on the base where the wiring is: {o}");
    assert!(rows[0].contains("System.md"), "{o}");
    // a variant that adds its own error is reported for that error only
    let worse = "---\ntype: PartDef\nname: Derived\nsupertype: System\nconnections:\n  - {from: tx.out, to: rx.missing}\n---\nA variant.\n";
    let o = validate(&[("Derived", worse)]);
    let derived: Vec<&str> = o.lines().filter(|l| l.contains("E931") && l.contains("Derived.md")).collect();
    assert_eq!(derived.len(), 1, "{o}");
    assert!(derived[0].contains("rx.missing"), "{o}");
}
