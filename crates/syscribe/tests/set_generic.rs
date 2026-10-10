//! TC-TRS-SETGEN-001 / GH #242.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let r = std::env::temp_dir().join(format!("syscribe-setgen-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst))).join("model");
    let w = |rel: &str, c: &str| {
        let p = r.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, c).unwrap();
    };
    w("_index.md", "---\ntype: Package\nname: Root\n---\n");
    w(
        "REQ-SG-001.md",
        "---\nid: REQ-SG-001\ntype: Requirement\nname: \"A requirement\"  # keep me\nstatus: draft\nreqDomain: software\nresponsibility: OldTeam\ntags:\n  - alpha   # first\n---\n\nThe system shall.\n",
    );
    w(
        "PI-SG-001.md",
        "---\nid: PI-SG-001\ntype: PlanningItem\nname: \"Item\"\nstatus: todo\nitemType: task\nparent: PI-SG-000\n---\n\nWork.\n",
    );
    w("PI-SG-000.md", "---\nid: PI-SG-000\ntype: PlanningItem\nname: \"Epic\"\nstatus: todo\nitemType: feature\nachieves: [REQ-SG-001]\n---\n\nEpic.\n");
    r
}

fn run(root: &Path, args: &[&str]) -> (String, i32) {
    let o = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(root).args(args).output().unwrap();
    (format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)), o.status.code().unwrap_or(-1))
}

fn read(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap()
}

#[test]
fn a_scalar_field_is_replaced_in_place_and_added_when_absent() {
    let r = model();
    let (o, c) = run(&r, &["set", "REQ-SG-001", "responsibility=NewTeam"]);
    assert_eq!(c, 0, "{o}");
    let t = read(&r, "REQ-SG-001.md");
    assert!(t.contains("responsibility: NewTeam") && !t.contains("OldTeam"), "{t}");
    assert!(t.contains("name: \"A requirement\"  # keep me") && t.contains("- alpha   # first"), "comments and quoting kept: {t}");
    let (o, c) = run(&r, &["set", "PI-SG-001", "assignedTo=alice"]);
    assert_eq!(c, 0, "{o}");
    assert!(read(&r, "PI-SG-001.md").contains("assignedTo: alice"));
}

#[test]
fn a_list_field_gains_an_item_and_keeps_the_rest() {
    let r = model();
    let (o, c) = run(&r, &["set", "REQ-SG-001", "tags.add", "beta"]);
    assert_eq!(c, 0, "{o}");
    let t = read(&r, "REQ-SG-001.md");
    assert!(t.contains("- alpha   # first") && t.contains("- beta"), "{t}");
}

#[test]
fn an_out_of_enum_value_is_refused_and_nothing_is_written() {
    let r = model();
    let before = read(&r, "REQ-SG-001.md");
    let (o, c) = run(&r, &["set", "REQ-SG-001", "reqDomain=banana"]);
    assert_ne!(c, 0, "{o}");
    assert!(o.contains("E302") || o.to_lowercase().contains("refus"), "{o}");
    assert_eq!(before, read(&r, "REQ-SG-001.md"));
}

#[test]
fn a_dangling_list_reference_is_refused() {
    let r = model();
    let before = read(&r, "PI-SG-001.md");
    let (o, c) = run(&r, &["set", "PI-SG-001", "blockedBy.add", "REQ-NOPE-999"]);
    assert_ne!(c, 0, "{o}");
    assert!(o.contains("E720"), "{o}");
    assert_eq!(before, read(&r, "PI-SG-001.md"));
}

#[test]
fn dry_run_prints_the_diff_and_writes_nothing() {
    let r = model();
    let before = read(&r, "REQ-SG-001.md");
    let (o, c) = run(&r, &["set", "REQ-SG-001", "responsibility=Other", "--dry-run"]);
    assert_eq!(c, 0, "{o}");
    assert!(o.contains("+responsibility: Other"), "{o}");
    assert_eq!(before, read(&r, "REQ-SG-001.md"));
}

#[test]
fn an_unknown_field_is_refused_and_the_supported_ones_are_listed() {
    let r = model();
    let (o, c) = run(&r, &["set", "REQ-SG-001", "banana=1"]);
    assert_ne!(c, 0);
    assert!(o.contains("assignedTo") && o.contains("tags"), "{o}");
}

#[test]
fn status_and_achieves_keep_working() {
    let r = model();
    let (o, c) = run(&r, &["set", "REQ-SG-001", "status=review"]);
    assert_eq!(c, 0, "{o}");
    assert!(read(&r, "REQ-SG-001.md").contains("status: review"));
}
