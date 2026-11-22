//! GH #516 / #509: an `ActionDef` made only of `SendAction`s (no `variables:`, no assignment) sends its messages in
//! `simulate` as a transition effect, and the composite checks (W936) see those sends.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const BEEP_ONLY: &str = "---\ntype: ActionDef\nname: Beep\nsubActions:\n  - {name: tx0, kind: SendAction, payload: Cmd, via: out}\n---\n";
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
fn mutate_seeds_the_loss_of_a_send_inside_an_effect_activity_and_the_scenario_kills_it() {
    // GH #510
    let (c, t) = run(&model(BEEP_ONLY), &["mutate", "Sys", "--json"]);
    assert_eq!(c, 0, "{t}");
    let j: serde_json::Value = serde_json::from_str(&t).unwrap_or_else(|e| panic!("{e}: {t}"));
    let lost: Vec<&serde_json::Value> = j["parts"].as_array().unwrap().iter().flat_map(|p| p["mutants"].as_array().unwrap()).filter(|m| m["operator"] == "remove-send").collect();
    assert_eq!(lost.len(), 1, "one send in one effect activity: {t}");
    assert_eq!(lost[0]["fate"], "killed", "the receiver never reaches Done without the message: {t}");
    let d = lost[0]["description"].as_str().unwrap();
    assert!(d.contains("tx0") && d.contains("Beep") && d.contains("Idle"), "attributed to the transition that runs it: {d}");
}

#[test]
fn a_scenario_that_never_checks_the_message_leaves_the_lost_send_alive() {
    let weak = "```gherkin\nFeature: system\n\n  Scenario: the tick is accepted\n    When tx receives Tick\n    Then tx is in state Idle\n```\n";
    let root = model(BEEP_ONLY);
    let tc = std::fs::read_to_string(root.join("TC-SYS-001.md")).unwrap();
    let head = tc.split("```gherkin").next().unwrap();
    std::fs::write(root.join("TC-SYS-001.md"), format!("{head}{weak}")).unwrap();
    let (_, t) = run(&root, &["mutate", "Sys", "--json"]);
    let j: serde_json::Value = serde_json::from_str(&t).unwrap_or_else(|e| panic!("{e}: {t}"));
    let fate = j["parts"].as_array().unwrap().iter().flat_map(|p| p["mutants"].as_array().unwrap()).find(|m| m["operator"] == "remove-send").map(|m| m["fate"].as_str().unwrap().to_string());
    assert_eq!(fate.as_deref(), Some("survived"), "{t}");
}

#[test]
fn w938_flags_payload_forms_that_are_ignored_and_names_the_working_one() {
    // GH #508
    for bad in ["arguments: [{name: phase, value: \"3\"}]", "values: {phase: 3}", "payload: {type: Cmd, phase: 3}"] {
        let beep = if bad.starts_with("payload") {
            format!("---\ntype: ActionDef\nname: Beep\nsubActions:\n  - {{name: tx0, kind: SendAction, {bad}, via: out}}\n---\n")
        } else {
            format!("---\ntype: ActionDef\nname: Beep\nsubActions:\n  - {{name: tx0, kind: SendAction, payload: Cmd, {bad}, via: out}}\n---\n")
        };
        let (_, t) = run(&model(&beep), &["validate"]);
        assert!(t.contains("W938") && t.contains("parameters: [{name:"), "{bad}: {t}");
    }
    let good = "---\ntype: ActionDef\nname: Beep\nsubActions:\n  - {name: tx0, kind: SendAction, payload: Cmd, via: out, parameters: [{name: phase, value: \"3\"}]}\n---\n";
    let (_, t) = run(&model(good), &["validate"]);
    assert!(!t.contains("W938"), "{t}");
}

#[test]
fn w938_flags_an_unknown_key_on_any_sub_action_kind() {
    let beep = "---\ntype: ActionDef\nname: Beep\nsubActions:\n  - {name: a, kind: AssignmentAction, assign: \"n := 1\", colour: red}\n---\n";
    let (_, t) = run(&model(beep), &["validate"]);
    assert!(t.contains("W938") && t.contains("`colour:`"), "{t}");
}
