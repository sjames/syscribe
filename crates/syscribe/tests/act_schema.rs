//! TC-TRS-ACTSCHEMA-001 / REQ-TRS-ACTSCHEMA-001 (GH #276): the executable action-flow schema and its checks.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn validate(body: &str) -> String {
    static N: AtomicU64 = AtomicU64::new(0);
    let root: PathBuf = std::env::temp_dir().join(format!("syscribe-actschema-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(root.join("A.md"), format!("---\ntype: ActionDef\nname: A\n{body}---\n")).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&root).arg("validate").output().unwrap();
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

fn has(out: &str, code: &str) -> bool {
    out.lines().any(|l| l.contains(code))
}

const GOOD: &str = "parameters:
  - {name: n, direction: in}
  - {name: result, direction: out}
variables:
  - {name: acc, value: \"0\"}
subActions:
  - {name: init, kind: AssignmentAction, assign: \"acc := n\"}
  - {name: big, kind: AssignmentAction, assign: \"result := acc * 2\"}
  - {name: small, kind: AssignmentAction, assign: \"result := acc\"}
controlNodes:
  - {name: d, kind: DecisionNode}
successionConnections:
  - {after: init, before: d}
  - {after: d, before: big, guard: \"acc > 10\"}
  - {after: d, before: small, guard: else}
";

fn with(replace_from: &str, replace_to: &str) -> String {
    assert!(GOOD.contains(replace_from), "{replace_from}");
    GOOD.replacen(replace_from, replace_to, 1)
}

#[test]
fn a_well_formed_executable_action_validates_clean() {
    let o = validate(GOOD);
    for c in ["E883", "E884", "E887", "W935", "W047"] {
        assert!(!has(&o, c), "{c} unexpected: {o}");
    }
}

#[test]
fn variables_and_assignments_reuse_the_state_machine_codes() {
    assert!(has(&validate(&with("value: \"0\"", "value: \"1 +\"")), "E883"), "bad initial value");
    assert!(has(&validate(&with("acc := n", "zz := n")), "E884"), "undeclared target");
    assert!(has(&validate(&with("acc := n", "acc = n")), "E884"), "not a statement");
}

#[test]
fn decision_wiring_is_checked() {
    let two_else = with("guard: \"acc > 10\"", "guard: else");
    assert!(has(&validate(&two_else), "E887"), "two else");
    let else_first = GOOD.replace("guard: \"acc > 10\"", "guard: TMP").replace("guard: else", "guard: \"acc > 10\"").replace("guard: TMP", "guard: else");
    assert!(has(&validate(&else_first), "E887"), "else not last");
    let none = "variables:\n  - {name: acc, value: \"0\"}\nsubActions:\n  - {name: init, kind: AssignmentAction, assign: \"acc := 1\"}\ncontrolNodes:\n  - {name: d, kind: DecisionNode}\nsuccessionConnections:\n  - {after: init, before: d}\n";
    assert!(has(&validate(none), "E887"), "a decision with no outgoing succession");
    let bad = with("guard: \"acc > 10\"", "guard: \"acc >> (\"");
    assert!(has(&validate(&bad), "E887"), "unparseable decision guard");
}

#[test]
fn loops_and_timed_accepts_are_checked() {
    let l = "variables:\n  - {name: i, value: \"0\"}\nsubActions:\n  - name: l\n    kind: LoopAction\n    loopKind: while\n    body:\n      - {name: inc, kind: AssignmentAction, assign: \"i := i + 1\"}\n";
    assert!(has(&validate(l), "E887"), "while without condition");
    let f = format!("{l}    condition: \"i < 3\"\n").replace("loopKind: while", "loopKind: for");
    assert!(has(&validate(&f), "E887"), "for is not supported");
    let ok = format!("{l}    condition: \"i < 3\"\n");
    assert!(!has(&validate(&ok), "E887"), "{}", validate(&ok));
    let acc = "variables:\n  - {name: i, value: \"0\"}\nsubActions:\n  - {name: w, kind: AcceptAction, after: \"2 kg\"}\n";
    assert!(has(&validate(acc), "E887"), "after must be a duration");
}

#[test]
fn guards_of_executable_actions_are_checked_for_the_expression_language() {
    let opaque = with("guard: \"acc > 10\"", "guard: \"the tank is full\"");
    assert!(has(&validate(&opaque), "E887"), "opaque decision guard is E887 (decision successions are always checked)");
    let unknown = GOOD.replace("{after: init, before: d}", "{after: init, before: d, guard: \"ghost > 1\"}");
    assert!(has(&validate(&unknown), "W935"), "name that is nothing");
    let ok = GOOD.replace("{after: init, before: d}", "{after: init, before: d, guard: \"n > 1 and big.result > 0\"}");
    assert!(!has(&validate(&ok), "W935"), "parameters and <subaction>.<pin> are known: {}", validate(&ok));
}

#[test]
fn existing_actions_see_nothing_new() {
    let plain = "subActions:\n  - {name: a, kind: Action}\n  - {name: b, kind: Action}\nsuccessionConnections:\n  - {after: a, before: b, guard: \"the door is open\"}\n";
    let o = validate(plain);
    for c in ["E883", "E884", "E887", "W935"] {
        assert!(!has(&o, c), "{c} on a non-executable action: {o}");
    }
}
