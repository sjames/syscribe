//! TC-TRS-STYPE-002 / REQ-TRS-STYPE-002: static type and dimension checking in executable action flows (E935).

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model(action: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-styact-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(root.join("A.md"), format!("---\ntype: ActionDef\nname: A\n{action}---\nAn action.\n")).unwrap();
    root
}

fn run(root: &PathBuf, args: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (o.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

const HEAD: &str = "parameters:
  - {name: level, direction: in}
  - {name: verdict, direction: out}
variables:
  - {name: margin, value: \"0\"}
  - {name: dist, value: \"1\", unit: \"SI::m\"}
  - {name: armed, value: \"false\"}
";

/// An action with one assignment sub-action `step`, and a decision after it with the given guard.
fn flow(assign: &str, guard: &str) -> String {
    format!(
        "{HEAD}subActions:\n  - {{name: step, kind: AssignmentAction, assign: \"{assign}\"}}\n  - {{name: hi, kind: AssignmentAction, assign: \"verdict := 1\"}}\n  - {{name: lo, kind: AssignmentAction, assign: \"verdict := 0\"}}\ncontrolNodes:\n  - {{name: d, kind: DecisionNode}}\nsuccessionConnections:\n  - {{after: step, before: d}}\n  - {{after: d, before: hi, guard: \"{guard}\"}}\n  - {{after: d, before: lo, guard: else}}\n"
    )
}

fn e935(action: &str) -> (bool, String) {
    let (_, t) = run(&model(action), &["validate"]);
    (t.contains("E935"), t)
}

#[test]
fn a_well_typed_action_is_clean() {
    let (bad, t) = e935(&flow("margin := level - 10; dist := dist * 2", "margin > 0 and not armed"));
    assert!(!bad, "{t}");
}

#[test]
fn guards_and_assignments_are_checked() {
    for (assign, guard, what) in [
        ("margin := 1", "margin", "a guard that is a number"),
        ("margin := 1", "dist < 5", "metres against a plain number"),
        ("dist := margin", "margin > 0", "assigning a number to a length"),
        ("armed := 1", "margin > 0", "assigning a number to a boolean"),
        ("margin := 1", "margin > 0 and dist", "and over a length"),
    ] {
        let (bad, t) = e935(&flow(assign, guard));
        assert!(bad, "{what} should be E935: {t}");
    }
}

#[test]
fn unknown_names_are_never_a_type_error() {
    // `level` has no default and `step.out` is a pin: both unknown
    let (bad, t) = e935(&flow("margin := level + 1; dist := dist + level", "step.out > 5 or level"));
    assert!(!bad, "{t}");
}

#[test]
fn nested_blocks_and_conditions_are_checked() {
    let a = format!("{HEAD}subActions:\n  - name: gate\n    kind: IfAction\n    condition: \"margin + 1\"\n    then:\n      - {{name: t, kind: AssignmentAction, assign: \"dist := margin\"}}\n    else:\n      - {{name: e, kind: AssignmentAction, assign: \"margin := 1\"}}\n  - name: again\n    kind: LoopAction\n    loopKind: while\n    condition: \"dist\"\n    body:\n      - {{name: b, kind: AssignmentAction, assign: \"margin := margin + dist\"}}\n");
    let (_, t) = run(&model(&a), &["validate"]);
    assert!(t.matches("E935").count() >= 4, "if condition, nested assignment, loop condition, nested sum: {t}");
}

#[test]
fn send_and_perform_arguments_are_checked() {
    let a = format!("{HEAD}subActions:\n  - {{name: tell, kind: SendAction, payload: Out, parameters: [{{name: x, value: \"dist + margin\"}}]}}\n");
    let (bad, t) = e935(&a);
    assert!(bad, "{t}");
}

#[test]
fn compile_refuses_what_validate_reports() {
    let root = model(&flow("margin := 1", "dist < 5"));
    let (c, t) = run(&root, &["compile", "A"]);
    assert_ne!(c, 0, "{t}");
    assert!(t.contains("E935"), "{t}");
}

#[test]
fn a_type_error_is_reported_with_another_lowering_diagnostic() {
    let a = flow("margin := 1", "dist < 5").replace("verdict := 0", "verdict := nowhere + 1");
    let (bad, t) = e935(&a);
    assert!(bad, "{t}");
}

fn model_with(files: &[(&str, &str)]) -> PathBuf {
    let root = model(HEAD);
    for (n, c) in files {
        std::fs::write(root.join(format!("{n}.md")), c).unwrap();
    }
    root
}

#[test]
fn a_callee_error_is_reported_on_the_callee_only_but_still_stops_compile() {
    let bad = format!("---\ntype: ActionDef\nname: Callee\n{}---\nCallee.\n", flow("margin := 1", "dist < 5"));
    let caller = "---\ntype: ActionDef\nname: Caller\nvariables:\n  - {name: z, value: \"0\"}\nsubActions:\n  - {name: go, kind: PerformAction, typedBy: Callee}\n  - {name: fin, kind: AssignmentAction, assign: \"z := 1\"}\nsuccessionConnections:\n  - {after: go, before: fin}\n---\nCaller.\n";
    let root = model_with(&[("Callee", &bad), ("Caller", caller)]);
    let (_, t) = run(&root, &["validate"]);
    let rows: Vec<&str> = t.lines().filter(|l| l.contains("E935")).collect();
    assert_eq!(rows.len(), 1, "one finding, on the callee: {t}");
    assert!(rows[0].contains("Callee.md"), "{t}");
    let (c, t) = run(&root, &["compile", "Caller"]);
    assert_ne!(c, 0, "{t}");
    assert!(t.contains("E935"), "the caller cannot be compiled with a broken callee: {t}");
}

#[test]
fn loop_bodies_accept_bindings_and_defaults_are_typed_correctly() {
    let a = format!("{HEAD}subActions:\n  - name: wait\n    kind: AcceptAction\n    payload: Msg\n    as: m\n  - name: again\n    kind: LoopAction\n    loopKind: while\n    condition: \"margin < 3 and m.ok\"\n    body:\n      - {{name: b, kind: AssignmentAction, assign: \"margin := margin + m.step\"}}\nsuccessionConnections:\n  - {{after: wait, before: again}}\n");
    let (bad, t) = e935(&a);
    assert!(!bad, "an accept binding is unknown: {t}");
    let defaulted = "parameters:\n  - {name: gain, direction: in, value: \"2\"}\n  - {name: lim, direction: in, value: \"5\", unit: \"SI::m\"}\nvariables:\n  - {name: d, value: \"1\", unit: \"SI::m\"}\nsubActions:\n  - {name: s, kind: AssignmentAction, assign: \"d := d * gain; d := d + lim\"}\n";
    let (bad, t) = e935(defaulted);
    assert!(!bad, "defaults give parameters their types, consistently: {t}");
    let wrong = defaulted.replace("d := d + lim", "d := d + gain");
    let (bad, t) = e935(&wrong);
    assert!(bad, "{t}");
}
