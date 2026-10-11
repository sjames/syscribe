//! REQ-TRS-ACTEXEC-001 (GH #276): a state machine runs executable ActionDefs named by its behaviour references.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::expr::{format_value, Value};
use syscribe_model::ir::lower;
use syscribe_model::sm_run::{Options, Run};
use syscribe_model::walker::walk_model;

fn machine(m_body: &str, actions: &[(&str, &str)]) -> syscribe_model::ir::Machine {
    static N: AtomicU64 = AtomicU64::new(0);
    let dir: PathBuf = std::env::temp_dir().join(format!("syscribe-smact-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(dir.join("M.md"), format!("---\ntype: StateDef\nname: M\n{m_body}---\n")).unwrap();
    for (n, b) in actions {
        std::fs::write(dir.join(format!("{n}.md")), format!("---\ntype: ActionDef\nname: {n}\n{b}---\n")).unwrap();
    }
    lower(&walk_model(&dir).unwrap(), "M").unwrap_or_else(|d| panic!("{d:?}"))
}

fn num(r: &Run, n: &str) -> String {
    format_value(&r.var(n).unwrap())
}

const INC: &str = "parameters:
  - {name: x, direction: in}
  - {name: y, direction: out}
subActions:
  - {name: calc, kind: AssignmentAction, assign: \"y := x + 1\"}
variables:
  - {name: unused, value: \"0\"}
";

const MACHINE: &str = "variables:
  - {name: x, value: \"4\"}
  - {name: y, value: \"0\"}
subStates:
  - name: A
    isInitial: true
    entryAction: Inc
    transitions:
      - {target: B, accept: Cmd::Go, effect: Inc}
  - name: B
";

#[test]
fn an_entry_action_that_names_an_executable_actiondef_runs_it_on_the_machines_variables() {
    let m = machine(MACHINE, &[("Inc", INC)]);
    let r = Run::new(&m, Options::default());
    assert!(r.error().is_none(), "{:?}", r.error());
    assert_eq!(num(&r, "y"), "5", "Inc read x = 4 and wrote y");
    let start = &r.trace()[0];
    assert!(start.actions.iter().any(|a| a.contains("Inc") && a.contains("executed")), "{:?}", start.actions);
}

#[test]
fn a_transition_effect_runs_it_with_the_current_values() {
    let m = machine(MACHINE, &[("Inc", INC)]);
    let mut r = Run::new(&m, Options::default());
    r.set("x", Value::Num(syscribe_model::expr::Quantity::scalar(10.0)));
    r.send("Cmd::Go", &[]);
    assert_eq!(r.active(), ["B"]);
    assert_eq!(num(&r, "y"), "11");
}

#[test]
fn a_callee_that_waits_or_mistypes_stops_the_run() {
    let waits = machine(MACHINE, &[("Inc", "variables:\n  - {name: v, value: \"0\"}\nsubActions:\n  - {name: w, kind: AcceptAction, payload: Pkg::Go}\n")]);
    let r = Run::new(&waits, Options::default());
    assert!(r.error().is_some_and(|e| e.contains("waits")), "{:?}", r.error());
    let bad = machine(MACHINE, &[("Inc", "parameters:\n  - {name: y, direction: out}\nvariables:\n  - {name: v, value: \"0\"}\nsubActions:\n  - {name: s, kind: AssignmentAction, assign: \"y := 1 s\"}\n")]);
    let r = Run::new(&bad, Options::default());
    assert!(r.error().is_some_and(|e| e.contains("'y'")), "a time written to a dimensionless variable: {:?}", r.error());
}

#[test]
fn a_non_executable_actiondef_stays_a_recorded_reference() {
    let m = machine(MACHINE, &[("Inc", "subActions:\n  - {name: a, kind: Action}\n")]);
    let r = Run::new(&m, Options::default());
    assert!(r.error().is_none());
    assert_eq!(num(&r, "y"), "0");
    assert!(r.trace()[0].actions.iter().any(|a| a.contains("behavior") && a.contains("Inc") && !a.contains("executed")), "{:?}", r.trace()[0].actions);
}

#[test]
fn the_machine_hash_covers_the_called_actions() {
    let a = machine(MACHINE, &[("Inc", INC)]);
    let b = machine(MACHINE, &[("Inc", &INC.replace("x + 1", "x + 2"))]);
    assert_ne!(a.hash, b.hash);
}
