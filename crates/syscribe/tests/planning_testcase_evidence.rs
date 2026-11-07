//! TC-TRS-PIEV-001 (GH #456): an `active` TestCase that no PlanningItem's `evidence: - ref:` names —
//! `W319` behind `[planning] require_testcase_evidence = true`, `W322` for a malformed `[planning]`
//! table, and the same set reported by `coverage --json`.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

struct Model {
    root: PathBuf,
}

impl Model {
    fn new(toml: &str) -> Model {
        static N: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!("syscribe-piev-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let m = Model { root };
        if !toml.is_empty() {
            m.write(".syscribe.toml", toml);
        }
        m.write("_index.md", "---\ntype: Package\nname: Root\n---\n");
        m.write("R/REQ-EV-001.md", "---\nid: REQ-EV-001\ntype: Requirement\nname: R1\nstatus: draft\nreqDomain: software\nreqClass: system\n---\n\nShall.\n");
        m.write("T/TC-EV-001.md", &tc("TC-EV-001", "active"));
        m.write("T/TC-EV-002.md", &tc("TC-EV-002", "active"));
        m.write("T/TC-EV-003.md", &tc("TC-EV-003", "active"));
        m.write("T/TC-EV-004.md", &tc("TC-EV-004", "draft"));
        // TC-EV-001 named by id, TC-EV-002 by qualified name; TC-EV-003 named by nobody.
        m.write(
            "P/PI-EV-001.md",
            "---\nid: PI-EV-001\ntype: PlanningItem\nname: Work\nstatus: in_progress\nachieves: [REQ-EV-001]\nevidence:\n  - ref: TC-EV-001\n  - ref: T::TC-EV-002\n---\n\nBody.\n",
        );
        m
    }
    fn write(&self, rel: &str, c: &str) {
        let p = self.root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    }
    fn run(&self, args: &[&str]) -> (String, String, i32) {
        let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(&self.root).args(args).output().unwrap();
        (String::from_utf8_lossy(&o.stdout).into(), String::from_utf8_lossy(&o.stderr).into(), o.status.code().unwrap_or(-1))
    }
    fn findings(&self, code: &str) -> Vec<String> {
        let (out, _e, _c) = self.run(&["validate", "--json"]);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}"));
        v.as_array()
            .into_iter()
            .flatten()
            .filter(|f| f["code"] == code)
            .map(|f| format!("{} {}", f["file"].as_str().unwrap_or(""), f["message"].as_str().unwrap_or("")))
            .collect()
    }
}
impl Drop for Model {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn tc(id: &str, status: &str) -> String {
    format!("---\nid: {id}\ntype: TestCase\nname: {id}\nstatus: {status}\ntestLevel: L2\nverifies: [REQ-EV-001]\n---\n\n```gherkin\nFeature: F\n\n  Scenario: S\n    Then it works\n```\n")
}

#[test]
fn without_the_policy_no_finding() {
    let m = Model::new("");
    assert!(m.findings("W319").is_empty());
    let m = Model::new("[planning]\nrequire_testcase_evidence = false\n");
    assert!(m.findings("W319").is_empty());
}

#[test]
fn an_active_testcase_no_item_names_is_warned() {
    let m = Model::new("[planning]\nrequire_testcase_evidence = true\n");
    let w = m.findings("W319");
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].contains("TC-EV-003"), "{w:?}");
    // camelCase spelling works too.
    let m = Model::new("[planning]\nrequireTestcaseEvidence = true\n");
    assert_eq!(m.findings("W319").len(), 1);
}

#[test]
fn a_malformed_planning_table_is_w322_and_ignored() {
    let m = Model::new("[planning]\nrequire_testcase_evidence = \"yes\"\nbogus = 1\n");
    let w = m.findings("W322");
    assert_eq!(w.len(), 2, "{w:?}");
    assert!(w.iter().any(|f| f.contains("bogus")), "{w:?}");
    assert!(w.iter().all(|f| f.contains(".syscribe.toml")), "{w:?}");
    assert!(m.findings("W319").is_empty());
    let m = Model::new("planning = 3\n");
    assert_eq!(m.findings("W322").len(), 1);
}

#[test]
fn coverage_json_lists_the_unplanned_testcases() {
    // Reported whether or not the warning is switched on.
    let m = Model::new("");
    let (out, e, c) = m.run(&["coverage", "--json"]);
    assert_eq!(c, 0, "{e}");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}"));
    let ids: Vec<&str> = v["testCasesWithoutPlanningEvidence"].as_array().unwrap().iter().filter_map(|t| t["id"].as_str()).collect();
    assert_eq!(ids, ["TC-EV-003"], "{out}");
    let (out, _e, c) = m.run(&["coverage"]);
    assert_eq!(c, 0);
    assert!(out.contains("TC-EV-003"), "{out}");
}

#[test]
fn the_baseline_accepts_the_existing_gap_and_fails_a_new_one() {
    let m = Model::new("[planning]\nrequire_testcase_evidence = true\n");
    let bl = m.root.join("accepted.json");
    let (o, e, c) = m.run(&["validate", "--write-baseline", bl.to_str().unwrap()]);
    assert_eq!(c, 0, "{o}{e}");
    let (out, _e, c) = m.run(&["validate", "--warnings-as-errors", "--baseline-file", bl.to_str().unwrap()]);
    assert_eq!(c, 0, "{out}");
    m.write("T/TC-EV-005.md", &tc("TC-EV-005", "active"));
    let (out, _e, c) = m.run(&["validate", "--warnings-as-errors", "--baseline-file", bl.to_str().unwrap()]);
    assert_eq!(c, 2, "{out}");
    assert!(out.contains("TC-EV-005"), "{out}");
}
