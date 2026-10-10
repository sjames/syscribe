//! TC-TRS-COVROLL-001 / GH #252 (matrix --rollup).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model(toml: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let d = std::env::temp_dir().join(format!("syscribe-roll-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = d.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    if !toml.is_empty() {
        w(".syscribe.toml", toml);
    }
    let req = |id: &str, class: &str, status: &str, parent: Option<&str>| {
        let d = parent.map(|p| format!("derivedFrom: [{p}]\n")).unwrap_or_default();
        format!("---\nid: {id}\ntype: Requirement\nname: {id}\nstatus: {status}\nreqDomain: software\nreqClass: {class}\ntags: [t]\n{d}---\n\nShall.\n")
    };
    let tc = |id: &str, status: &str, v: &str| format!("---\nid: {id}\ntype: TestCase\nname: {id}\nstatus: {status}\ntestLevel: L3\nverifies: [{v}]\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n");
    w("R/REQ-RL-001.md", &req("REQ-RL-001", "stakeholder", "approved", None));
    w("R/REQ-RL-002.md", &req("REQ-RL-002", "system", "approved", Some("REQ-RL-001")));
    w("R/REQ-RL-003.md", &req("REQ-RL-003", "system", "draft", Some("REQ-RL-001")));
    w("R/REQ-RL-004.md", &req("REQ-RL-004", "system", "approved", None));
    w("R/TC-RL-002.md", &tc("TC-RL-002", "active", "REQ-RL-002"));
    w("R/TC-RL-003.md", &tc("TC-RL-003", "draft", "REQ-RL-003"));
    d
}

fn run(d: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(d).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

#[test]
fn rows_verdicts_and_footer() {
    let d = model("");
    let (o, c) = run(&d, &["matrix", "--rollup"]);
    assert_eq!(c, 0, "{o}");
    let line = |id: &str| o.lines().find(|l| l.contains(id)).unwrap_or_else(|| panic!("{id}: {o}")).to_string();
    assert!(line("REQ-RL-001").contains("1/2 active, 1 planned") && line("REQ-RL-001").contains('◐'), "{o}");
    assert!(line("REQ-RL-002").contains("verified") && line("REQ-RL-002").contains('●'), "{o}");
    assert!(line("REQ-RL-003").contains("planned") && line("REQ-RL-003").contains('◐'), "{o}");
    assert!(line("REQ-RL-004").contains("uncovered") && line("REQ-RL-004").contains('○'), "{o}");
    assert!(o.contains("By reqClass") && o.contains("system") && o.contains("stakeholder"), "{o}");
}

#[test]
fn filters_policy_and_json() {
    let d = model("");
    let (o, _) = run(&d, &["matrix", "--rollup", "--status", "draft"]);
    assert!(o.contains("REQ-RL-003") && !o.contains("REQ-RL-002"), "{o}");
    let (j, c) = run(&d, &["matrix", "--rollup", "--json"]);
    assert_eq!(c, 0, "{j}");
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    let row = |id: &str| v["rows"].as_array().unwrap().iter().find(|r| r["id"] == id).unwrap().clone();
    assert_eq!(row("REQ-RL-001")["leavesActive"], 1);
    assert_eq!(row("REQ-RL-001")["rule"], "both");
    assert_eq!(row("REQ-RL-002")["verdict"], "complete");
    assert_eq!(row("REQ-RL-004")["verdict"], "none");
    assert_eq!(v["byClass"]["system"]["complete"], 1);
    // a rollup rule completes the stakeholder parent only when every leaf is verified: here one is only planned
    let toml = "[[coverage.rule]]\nreqClass = \"stakeholder\"\nparent_rule = \"rollup\"\n";
    let (j, _) = run(&model(toml), &["matrix", "--rollup", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&j).unwrap();
    let r1 = v["rows"].as_array().unwrap().iter().find(|r| r["id"] == "REQ-RL-001").unwrap().clone();
    assert_eq!((r1["rule"].as_str(), r1["verdict"].as_str()), (Some("rollup"), Some("partial")));
    // a configuration error is exit 1
    let (_, c) = run(&model("[coverage]\ndefault = \"x\"\n"), &["matrix", "--rollup"]);
    assert_eq!(c, 1);
}
