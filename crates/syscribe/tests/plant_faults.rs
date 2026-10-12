//! TC-TRS-PLANT-001 / REQ-TRS-PLANT-001 (GH #340): the fault kinds that act on an in-process plant — a stuck or offset
//! output, a plant failed silent and a changed parameter — injected from an FMEA row into a composite scenario.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const FILES: &[(&str, &str)] = &[
    ("CtlM", "---\ntype: StateDef\nname: CtlM\nvariables:\n  - {name: u, value: \"1\"}\n  - {name: y, value: \"0\"}\nsubStates:\n  - {name: Run, isInitial: true}\n---\n"),
    ("CtlP", "---\ntype: PartDef\nname: CtlP\nexhibitsStates: [CtlM]\n---\nController.\n"),
    ("Lag", "---\ntype: PlantDef\nname: Lag\nparameters:\n  - {name: k, value: \"1\"}\n  - {name: tau, value: \"200 ms\"}\ninputs:\n  - {name: u}\noutputs:\n  - {name: y}\nstates:\n  - {name: x, value: \"0\"}\nequations:\n  - \"der(x) = (k * u - x) / tau\"\n  - \"y := x\"\n---\nA first-order lag.\n"),
    ("LagP", "---\ntype: PartDef\nname: LagP\nplant: Lag\n---\nThe plant.\n"),
    ("Sys", "---\ntype: PartDef\nname: Sys\nplantStep: \"10 ms\"\nfeatures:\n  - {name: ctl, typedBy: CtlP}\n  - {name: lag, typedBy: LagP}\nbindingConnections:\n  - {from: ctl.u, to: lag.u}\n  - {from: lag.y, to: ctl.y}\n---\nThe system.\n"),
    ("REQ-SYS-001", "---\ntype: Requirement\nid: REQ-SYS-001\nname: response\nstatus: approved\nreqDomain: software\n---\n\nThe output shall follow the input.\n"),
    (
        "FMEA-LAG-001",
        "---
type: FMEASheet
id: FMEA-LAG-001
name: \"FMEA of the lag\"
status: draft
entries:
  - {id: FM-LAG-001, ref: LagP, failureMode: \"output stuck\", injection: [{kind: stuck, variable: y, value: \"0.5\"}]}
  - {id: FM-LAG-002, ref: LagP, failureMode: \"output biased\", injection: [{kind: offset, variable: y, by: \"0.25\"}]}
  - {id: FM-LAG-003, ref: LagP, failureMode: \"plant dead\", injection: [{kind: silent}]}
  - {id: FM-LAG-004, ref: LagP, failureMode: \"gain doubled\", injection: [{kind: parameter, parameter: k, value: \"2\"}]}
  - {id: FM-LAG-005, ref: LagP, failureMode: \"no such parameter\", injection: [{kind: parameter, parameter: zeta, value: \"2\"}]}
  - {id: FM-LAG-006, ref: LagP, failureMode: \"no such output\", injection: [{kind: stuck, variable: ghost, value: \"2\"}]}
---
",
    ),
];

const TC: &str = "---
type: TestCase
id: TC-SYS-001
name: plant under faults
status: active
testLevel: L3
subject: Sys
verifies: [REQ-SYS-001]
---

```gherkin
Feature: plant faults

  Scenario: healthy
    When 1 s elapses
    Then ctl.y > 0.9932 and ctl.y < 0.9933

  Scenario: a stuck output holds from the moment it is injected
    When 1 s elapses
    And fault FM-LAG-001 is injected
    Then ctl.y == 0.5
    When 1 s elapses
    Then ctl.y == 0.5
    When fault FM-LAG-001 is cleared
    Then ctl.y > 0.99

  Scenario: an offset output
    Given fault FM-LAG-002 is injected
    When 1 s elapses
    Then ctl.y > 1.2432 and ctl.y < 1.2433

  Scenario: a dead plant is no longer read
    When 100 ms elapses
    And fault FM-LAG-003 is injected
    And 1 s elapses
    Then ctl.y > 0.39 and ctl.y < 0.40

  Scenario: a changed parameter, restored when cleared
    Given fault FM-LAG-004 is injected
    When 1 s elapses
    Then ctl.y > 1.986 and ctl.y < 1.987
    When fault FM-LAG-004 is cleared
    And 2 s elapses
    Then ctl.y < 1.001
```
";

fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-plantfault-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    for (n, c) in FILES {
        std::fs::write(root.join(format!("{n}.md")), c).unwrap();
    }
    std::fs::write(root.join("TC-SYS-001.md"), TC).unwrap();
    root
}

fn run(root: &PathBuf, args: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (o.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

#[test]
fn every_plant_fault_kind_behaves_as_specified() {
    let root = model();
    let (c, t) = run(&root, &["simulate", "TC-SYS-001", "--json"]);
    let j: serde_json::Value = serde_json::from_str(&t).unwrap_or_else(|e| panic!("{e}: {t}"));
    let v = j["verdicts"].as_array().unwrap();
    let failed: Vec<String> = v.iter().filter(|x| x["status"] != "pass").map(|x| format!("{}: {}", x["scenario"], x["message"])).collect();
    assert!(failed.is_empty() && c == 0, "{failed:?}\n{t}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_fault_that_does_not_fit_the_plant_is_reported_when_injected() {
    let root = model();
    let tc = TC.split("```gherkin").next().unwrap().to_string()
        + "```gherkin\nFeature: bad faults\n\n  Scenario: an unknown parameter\n    Given fault FM-LAG-005 is injected\n    Then ctl.y == 0\n\n  Scenario: an unknown output\n    Given fault FM-LAG-006 is injected\n    Then ctl.y == 0\n```\n";
    std::fs::write(root.join("TC-SYS-001.md"), tc).unwrap();
    let (c, t) = run(&root, &["simulate", "TC-SYS-001", "--json"]);
    assert_ne!(c, 0, "{t}");
    assert!(t.contains("'zeta' is not a parameter of its plant"), "{t}");
    assert!(t.contains("'ghost' is not a variable of its machine or an output of its plant"), "{t}");
    // nothing about the composite needs FMI support: no I011
    let (_, v) = run(&root, &["validate"]);
    assert!(!v.contains("I011"), "{v}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn stacked_parameter_faults_and_a_stuck_silent_plant_do_not_depend_on_order() {
    let root = model();
    let sheet = std::fs::read_to_string(root.join("FMEA-LAG-001.md")).unwrap().replace(
        "  - {id: FM-LAG-005,",
        "  - {id: FM-LAG-007, ref: LagP, failureMode: \"gain tripled\", injection: [{kind: parameter, parameter: k, value: \"3\"}]}\n  - {id: FM-LAG-005,",
    );
    std::fs::write(root.join("FMEA-LAG-001.md"), sheet).unwrap();
    let tc = TC.split("```gherkin").next().unwrap().to_string()
        + "```gherkin\nFeature: order\n\n  Scenario: the first of two faults on one parameter is cleared first\n    Given fault FM-LAG-004 is injected\n    And fault FM-LAG-007 is injected\n    When fault FM-LAG-004 is cleared\n    And 2 s elapses\n    Then ctl.y > 2.99\n    When fault FM-LAG-007 is cleared\n    And 3 s elapses\n    Then ctl.y > 0.999 and ctl.y < 1.001\n\n  Scenario: a silent plant then a stuck output\n    When 100 ms elapses\n    And fault FM-LAG-003 is injected\n    And fault FM-LAG-001 is injected\n    And 100 ms elapses\n    Then ctl.y == 0.5\n\n  Scenario: a stuck output then a silent plant\n    When 100 ms elapses\n    And fault FM-LAG-001 is injected\n    And fault FM-LAG-003 is injected\n    And 100 ms elapses\n    Then ctl.y == 0.5\n```\n";
    std::fs::write(root.join("TC-SYS-001.md"), tc).unwrap();
    let (c, t) = run(&root, &["simulate", "TC-SYS-001", "--json"]);
    let j: serde_json::Value = serde_json::from_str(&t).unwrap_or_else(|e| panic!("{e}: {t}"));
    let failed: Vec<String> = j["verdicts"].as_array().unwrap().iter().filter(|x| x["status"] != "pass").map(|x| format!("{}: {}", x["scenario"], x["message"])).collect();
    assert!(failed.is_empty() && c == 0, "{failed:?}\n{t}");
    let _ = std::fs::remove_dir_all(&root);
}
