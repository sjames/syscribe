//! `REQ-TRS-PLANCOV-001` (ADR-SYS-COVERAGE-001, GH #500/#501, slice 1): a TestPlan declares the required
//! coverage — `conditions:`, `minUnits:`, `mode:`, `points:` — and `validate` checks the declaration.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model(plan_extra: &str, plan_type: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-plancov-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w("REQ-HTR-001.md", "---\nid: REQ-HTR-001\ntype: Requirement\nname: \"Heater response\"\nstatus: approved\nreqDomain: system\nreqClass: system\n---\n\nThe heater shall reach the setpoint.\n");
    w("TC-HTR-001.md", "---\nid: TC-HTR-001\ntype: TestCase\ntestLevel: L4\nstatus: active\nname: \"Heater response\"\nverifies: [REQ-HTR-001]\n---\n\n```gherkin\nFeature: F\n  Scenario: S\n    Given a heater\n```\n");
    let id = if plan_type == "TestPlan" { "TP-HTR-001" } else { "REQ-HTR-002" };
    let extra_req = if plan_type == "TestPlan" { "" } else { "reqDomain: system\nreqClass: system\n" };
    w(&format!("{id}.md"), &format!("---\ntype: {plan_type}\nid: {id}\nname: \"Heater plan\"\nstatus: draft\n{extra_req}testCases: [TC-HTR-001]\n{plan_extra}---\n\nPlan.\n"));
    root
}

fn validate(root: &PathBuf) -> String {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).arg("validate").output().unwrap();
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

const AMB: &str = "conditions:\n  ambient:\n    - {name: cold, value: -20 degC}\n    - {name: nominal, value: 23 degC}\n    - {name: hot, value: 70 degC}\n";

#[test]
fn a_well_formed_declaration_is_accepted_without_w047() {
    let out = validate(&model(&format!("{AMB}minUnits: 2\nmode: product\n"), "TestPlan"));
    for code in ["E607", "E608", "W047"] {
        assert!(!out.contains(code), "{code} raised:\n{out}");
    }
}

#[test]
fn explicit_points_are_accepted() {
    let out = validate(&model(&format!("{AMB}points:\n  - {{ambient: hot}}\n  - {{ambient: cold}}\n"), "TestPlan"));
    assert!(!out.contains("E607") && !out.contains("E608"), "{out}");
}

#[test]
fn a_malformed_declaration_is_e607() {
    for bad in ["minUnits: 0\n", "minUnits: two\n", "conditions:\n  ambient: hot\n", "conditions:\n  ambient: [hot, hot]\n", "mode: product\n", &format!("{AMB}mode: both\n")] {
        let out = validate(&model(bad, "TestPlan"));
        assert!(out.contains("E607"), "no E607 for {bad:?}:\n{out}");
    }
}

#[test]
fn points_and_mode_together_are_e607() {
    let out = validate(&model(&format!("{AMB}mode: each\npoints:\n  - {{ambient: hot}}\n"), "TestPlan"));
    assert!(out.contains("E607") && out.contains("exclusive"), "{out}");
}

#[test]
fn a_point_that_names_nothing_declared_is_e608() {
    let out = validate(&model(&format!("{AMB}points:\n  - {{ambient: tepid}}\n"), "TestPlan"));
    assert!(out.contains("E608") && out.contains("tepid"), "{out}");
    let out = validate(&model(&format!("{AMB}points:\n  - {{humidity: wet}}\n"), "TestPlan"));
    assert!(out.contains("E608") && out.contains("humidity"), "{out}");
}

#[test]
fn on_any_other_type_the_keys_stay_unrecognised() {
    let out = validate(&model("minUnits: 2\n", "Requirement"));
    assert!(out.contains("W047") && out.contains("minUnits"), "{out}");
    assert!(!out.contains("E607"), "the checks apply to a TestPlan only:\n{out}");
}
