//! TC-TRS-FMI-001 / REQ-TRS-FMI-001 (GH #291): FMU plants on the command line. The first module runs in the
//! default build and checks the gating; the second needs `--features fmi`.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const CTL: &str = "---
type: StateDef
name: Ctl
variables:
  - {name: height, value: \"0\", unit: \"SI::m\"}
subStates:
  - name: Falling
    isInitial: true
    transitions:
      - {target: Bounced, accept: {when: \"height < 0.1 m\"}}
  - name: Bounced
---
";

const TC: &str = "---
type: TestCase
id: TC-FMI-001
name: ball
status: active
testLevel: L2
subject: Ctl
verifies: [REQ-FMI-001]
plants: [Ball]
plantStep: \"10 ms\"
plantConnections:
  - {from: Ball.h, to: height}
---

```gherkin
Feature: ball

  Scenario: falls and bounces
    When 0.3 s elapses
    Then Ctl is in state Falling
    When 0.3 s elapses
    Then Ctl is in state Bounced
    And height < 0.5 m

  Scenario: starts again from the top
    Then height == 1 m
    When 0.1 s elapses
    Then Ctl is in state Falling
```
";

fn part(sha: &str) -> String {
    format!("---\ntype: Part\nname: Ball\nfmu:\n  file: BouncingBall-2.0.fmu\n  fmiVersion: \"2.0\"\n  kind: cosimulation\n  sha256: {sha}\n  parameters: {{e: 0.7}}\n---\n")
}

fn model(sha: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let root = std::env::temp_dir().join(format!("syscribe-fmicli-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("_index.md"), "---\ntype: Package\nname: Root\n---\n").unwrap();
    std::fs::write(root.join("Ctl.md"), CTL).unwrap();
    std::fs::write(root.join("REQ-FMI-001.md"), "---\ntype: Requirement\nid: REQ-FMI-001\nname: ball\nstatus: approved\nreqDomain: software\n---\n\nThe ball shall bounce.\n").unwrap();
    std::fs::write(root.join("Ball.md"), part(sha)).unwrap();
    std::fs::write(root.join("TC-FMI-001.md"), TC).unwrap();
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../syscribe-fmi/tests/fmus/BouncingBall-2.0.fmu");
    std::fs::copy(src, root.join("BouncingBall-2.0.fmu")).unwrap();
    root
}

fn run(root: &PathBuf, args: &[&str]) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (o.status.code().unwrap_or(-1), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
}

const GOOD_SHA: &str = "1604856668b4c6081109e2ef0b778ef725a285095373bd7a54b24c7a9d6faad4";

#[cfg(not(feature = "fmi"))]
mod without_the_feature {
    use super::*;

    #[test]
    fn validate_explains_with_i011_and_checks_nothing() {
        let root = model("wrong-hash-is-not-checked");
        let (c, t) = run(&root, &["validate"]);
        assert_eq!(c, 0, "{t}");
        assert!(t.contains("I011") && t.contains("--features fmi"), "{t}");
        assert!(!t.contains("E966") && !t.contains("W047"), "{t}");
    }

    #[test]
    fn simulate_of_a_plant_testcase_exits_1_with_the_reason() {
        let root = model(GOOD_SHA);
        let (c, t) = run(&root, &["simulate", "TC-FMI-001"]);
        assert_eq!(c, 1, "{t}");
        assert!(t.contains("no FMI support") && t.contains("--features fmi"), "{t}");
    }

    #[test]
    fn the_rest_of_the_tool_is_unaffected() {
        let root = model(GOOD_SHA);
        let (c, _) = run(&root, &["compile", "Ctl"]);
        assert_eq!(c, 0);
    }
}

#[cfg(feature = "fmi")]
mod with_the_feature {
    use super::*;

    #[test]
    fn validate_checks_the_fmu_and_the_wiring() {
        let (c, t) = run(&model(GOOD_SHA), &["validate"]);
        assert_eq!(c, 0, "{t}");
        assert!(!t.contains("I011") && !t.contains("E966") && !t.contains("E889"), "{t}");
        let (c, t) = run(&model("0000"), &["validate"]);
        assert_eq!(c, 1, "{t}");
        assert!(t.contains("E966") && t.contains("sha256"), "{t}");
    }

    #[test]
    fn scenarios_run_against_the_real_fmu_with_fresh_plants_each() {
        let root = model(GOOD_SHA);
        let (c, t) = run(&root, &["simulate", "TC-FMI-001", "--json"]);
        assert_eq!(c, 0, "{t}");
        let j: serde_json::Value = serde_json::from_str(&t).unwrap();
        let v = j["verdicts"].as_array().unwrap();
        assert!(v.iter().all(|x| x["status"] == "pass"), "{t}");
        assert_eq!(v[0]["plants"][0]["plant"]["fmiVersion"], "2.0");
        assert_eq!(v[0]["plants"][0]["plant"]["sha256"], GOOD_SHA);
        assert_eq!(v[1]["status"], "pass", "the second scenario starts from h = 1 m again");
    }

    #[test]
    fn record_keeps_the_plant_provenance_beside_the_trace() {
        let root = model(GOOD_SHA);
        let (c, t) = run(&root, &["simulate", "TC-FMI-001", "--record", "--run", "r1"]);
        assert_eq!(c, 0, "{t}");
        let dir = root.join(".syscribe/traces");
        let any = std::fs::read_dir(&dir).unwrap().next().unwrap().unwrap().path();
        let j: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(any).unwrap()).unwrap();
        assert_eq!(j["plants"][0]["plant"]["sha256"], GOOD_SHA);
        assert!(j["plants"][0]["plant"]["platform"].is_string() && j["plants"][0]["plant"]["backend"].is_string());
    }

    #[test]
    fn a_tampered_fmu_stops_the_run_before_it_starts() {
        let root = model("deadbeef");
        let (c, t) = run(&root, &["simulate", "TC-FMI-001"]);
        assert_ne!(c, 0, "{t}");
    }

    #[test]
    fn coverage_works_with_plants_and_mutation_is_refused() {
        let root = model(GOOD_SHA);
        let (c, t) = run(&root, &["simulate", "Ctl", "--all", "--coverage", "--json"]);
        assert_eq!(c, 0, "{t}");
        let j: serde_json::Value = serde_json::from_str(&t).unwrap();
        assert_eq!(j["coverage"]["states"]["covered"], 2, "{t}");
        let (c, t) = run(&root, &["mutate", "Ctl"]);
        assert_eq!(c, 1, "{t}");
        assert!(t.contains("plants"), "{t}");
    }
}
