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
