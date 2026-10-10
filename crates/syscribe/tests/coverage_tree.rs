//! TC-TRS-COVTREE-001 / GH #252.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model(full: bool) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-covtree-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = r.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    let req = |id: &str, parent: Option<&str>| {
        let d = parent.map(|p| format!("derivedFrom: [{p}]\n")).unwrap_or_default();
        format!("---\nid: {id}\ntype: Requirement\nname: \"{id}\"\nstatus: approved\nreqDomain: software\nreqClass: system\n{d}---\n\nShall.\n")
    };
    let tc = |id: &str, status: &str, v: &str| {
        format!("---\nid: {id}\ntype: TestCase\nname: \"{id}\"\nstatus: {status}\ntestLevel: L3\nverifies: [{v}]\n---\n\n```gherkin\nFeature: f\n  Scenario: s\n    Then ok\n```\n")
    };
    w("REQ-CT-001.md", &req("REQ-CT-001", None));
    w("REQ-CT-002.md", &req("REQ-CT-002", Some("REQ-CT-001")));
    w("REQ-CT-003.md", &req("REQ-CT-003", Some("REQ-CT-001")));
    w("REQ-CT-004.md", &req("REQ-CT-004", Some("REQ-CT-001")));
    w("TC-CT-001.md", &tc("TC-CT-001", "active", "REQ-CT-002"));
    if full {
        w("TC-CT-002.md", &tc("TC-CT-002", "active", "REQ-CT-003"));
        w("TC-CT-003.md", &tc("TC-CT-003", "active", "REQ-CT-004"));
        w("TC-CT-004.md", &tc("TC-CT-004", "active", "REQ-CT-001"));
    } else {
        w("TC-CT-002.md", &tc("TC-CT-002", "draft", "REQ-CT-003"));
    }
    r
}

fn run(root: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (String::from_utf8_lossy(&o.stdout).into_owned(), o.status.code().unwrap_or(-1))
}

#[test]
fn mixed_tree_aggregates_leaves_and_marks_the_parent_partial() {
    let r = model(false);
    let (o, c) = run(&r, &["coverage", "tree", "REQ-CT-001"]);
    assert_eq!(c, 0, "{o}");
    let top = o.lines().next().unwrap();
    assert!(top.starts_with("◐ REQ-CT-001"), "{o}");
    assert!(top.contains("leaves 1/3 active") && top.contains("1 planned") && top.contains("direct tests 0"), "{o}");
    assert!(o.contains("● REQ-CT-002"), "{o}");
    assert!(o.contains("◐ REQ-CT-003"), "{o}");
    assert!(o.contains("○ REQ-CT-004"), "{o}");
}

#[test]
fn fully_covered_parent_with_a_direct_test_is_filled() {
    let r = model(true);
    let (o, c) = run(&r, &["coverage", "tree", "REQ-CT-001"]);
    assert_eq!(c, 0, "{o}");
    assert!(o.lines().next().unwrap().starts_with("● REQ-CT-001"), "{o}");
    assert!(o.lines().next().unwrap().contains("leaves 3/3 active"), "{o}");
}

#[test]
fn covered_only_through_children_stays_partial() {
    let r = model(true);
    std::fs::remove_file(r.join("TC-CT-004.md")).unwrap();
    let (o, _) = run(&r, &["coverage", "tree", "REQ-CT-001"]);
    assert!(o.lines().next().unwrap().starts_with("◐ REQ-CT-001"), "{o}");
    assert!(o.contains("direct tests 0"), "{o}");
}

#[test]
fn json_carries_counts_and_children_and_unknown_root_fails() {
    let r = model(false);
    let (o, c) = run(&r, &["coverage", "tree", "REQ-CT-001", "--json"]);
    assert_eq!(c, 0, "{o}");
    let v: serde_json::Value = serde_json::from_str(&o).unwrap();
    assert_eq!(v["id"], "REQ-CT-001");
    assert_eq!(v["leavesActive"], 1);
    assert_eq!(v["leavesPlanned"], 1);
    assert_eq!(v["leavesUncovered"], 1);
    assert_eq!(v["children"].as_array().unwrap().len(), 3);
    let (_, c) = run(&r, &["coverage", "tree", "REQ-NOPE-999"]);
    assert_eq!(c, 1);
    let (_, c) = run(&r, &["coverage", "bogus"]);
    assert_eq!(c, 1);
}

fn base(extra: &[(&str, String)]) -> PathBuf {
    let r = model(true);
    for (f, c) in extra {
        std::fs::write(r.join(f), c).unwrap();
    }
    r
}

fn req_with(id: &str, parents: &[&str]) -> String {
    let d = if parents.is_empty() { String::new() } else { format!("derivedFrom: [{}]\n", parents.join(", ")) };
    format!("---\nid: {id}\ntype: Requirement\nname: \"{id}\"\nstatus: approved\nreqDomain: software\nreqClass: system\n{d}---\n\nShall.\n")
}

#[test]
fn a_shared_descendant_is_counted_once_and_the_tree_does_not_depend_on_id_order() {
    // 002 is a child of both 001 and 003; 003 is a child of 001 (the sort-order trap).
    let r = base(&[("REQ-CT-003.md", req_with("REQ-CT-003", &["REQ-CT-001"])), ("REQ-CT-002.md", req_with("REQ-CT-002", &["REQ-CT-001", "REQ-CT-003"]))]);
    let (o, _) = run(&r, &["coverage", "tree", "REQ-CT-001", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&o).unwrap();
    // distinct leaves: 002 (verified), 004 (verified) — 003 is a parent of 002
    assert_eq!(v["leavesActive"], 2, "{o}");
    let kids = v["children"].as_array().unwrap();
    let k3 = kids.iter().find(|k| k["id"] == "REQ-CT-003").unwrap();
    assert_eq!(k3["leaf"], false, "{o}");
    assert_eq!(k3["children"].as_array().unwrap().len(), 1, "{o}");
}

#[test]
fn a_cycle_terminates_and_a_draft_only_direct_test_does_not_complete_a_parent() {
    let r = base(&[("REQ-CT-001.md", req_with("REQ-CT-001", &["REQ-CT-002"]))]);
    let (o, c) = run(&r, &["coverage", "tree", "REQ-CT-001"]);
    assert_eq!(c, 0, "{o}");
    // draft-only direct test on the parent: all leaves verified but no active direct test
    let r = model(true);
    let t = std::fs::read_to_string(r.join("TC-CT-004.md")).unwrap().replace("status: active", "status: draft");
    std::fs::write(r.join("TC-CT-004.md"), t).unwrap();
    let (o, _) = run(&r, &["coverage", "tree", "REQ-CT-001"]);
    assert!(o.lines().next().unwrap().starts_with("◐ REQ-CT-001"), "{o}");
    assert!(o.lines().next().unwrap().contains("direct tests 0"), "{o}");
}

fn with_plan(root: &Path, test_cases: &str) {
    let p = root.join("TP-CT-001.md");
    std::fs::write(p, format!("---\ntype: TestPlan\nid: TP-CT-001\nname: p\nstatus: approved\nscope: integration\ntestCases: [{test_cases}]\n---\n\nPlan.\n")).unwrap();
}

#[test]
fn plan_lens_counts_only_the_plans_tests() {
    let r = model(true);
    let (all, _) = run(&r, &["coverage", "tree", "REQ-CT-001"]);
    assert!(all.lines().next().unwrap().contains("leaves 3/3 active"), "{all}");
    with_plan(&r, "TC-CT-001, TC-CT-004");
    let (o, c) = run(&r, &["coverage", "tree", "REQ-CT-001", "--plan", "TP-CT-001"]);
    assert_eq!(c, 0, "{o}");
    let first = o.lines().next().unwrap();
    assert!(first.contains("leaves 1/") && first.contains("active"), "only TC-CT-001's leaf is verified under the plan: {o}");
    // flag order does not matter, and the lens reaches --json
    let (j, c) = run(&r, &["coverage", "tree", "--plan", "TP-CT-001", "--json", "REQ-CT-001"]);
    let (j2, c2) = run(&r, &["coverage", "tree", "REQ-CT-001", "--plan", "TP-CT-001", "--json"]);
    assert_eq!((c, c2), (0, 0), "{j}{j2}");
    let v: serde_json::Value = serde_json::from_str(&j2).unwrap();
    assert_eq!(v["leavesActive"], 1, "{j2}");
    assert!(first.contains("direct tests 1"), "TC-CT-004 is in the plan: {o}");
}

#[test]
fn a_root_outside_the_lens_or_an_unknown_plan_exits_one() {
    let r = model(true);
    with_plan(&r, "TC-CT-001");
    let (_, c) = run(&r, &["coverage", "tree", "REQ-CT-001", "--plan", "TP-NOPE"]);
    assert_eq!(c, 1);
    let (o, c) = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m").arg(&r).args(["coverage", "tree", "REQ-CT-003", "--plan", "TP-CT-001"]).output()
        .map(|o| (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))).unwrap();
    assert_eq!(c, 1, "{o}");
    assert!(o.contains("does not resolve") && o.contains("TP-CT-001"), "names the lens: {o}");
}

#[test]
fn an_unknown_configuration_exits_one() {
    let r = model(true);
    let (_, c) = run(&r, &["coverage", "tree", "REQ-CT-001", "--config", "CONF-NOPE"]);
    assert_eq!(c, 1);
}
