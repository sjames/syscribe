//! Regression tests for `syscribe set` (issue #112): schema-aware mutation of
//! a narrow allowlist of fields (`status=`, `evidence.add`, `achieves.add`),
//! each validated before anything is written.
//!
//! Black-box: builds a minimal, self-contained model in a fresh temp
//! directory per test and drives the `syscribe` binary against it.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn new_model() -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let root = std::env::temp_dir()
        .join(format!("syscribe-set-{}-{}-{}", std::process::id(), nanos, n))
        .join("model");
    std::fs::create_dir_all(&root).unwrap();
    write(&root, "_index.md", "---\ntype: Package\nname: SetCmd\n---\n\nModel root.\n");
    root
}

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn read(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap()
}

fn run(root: &Path, args: &[&str]) -> (String, String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m")
        .arg(root)
        .args(args)
        .output()
        .expect("spawn syscribe");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

const REQ_MD: &str = "---\nid: REQ-SET-001\ntype: Requirement\nname: \"A requirement\"\nstatus: draft\nreqDomain: system\n---\n\nBody.\n";

#[test]
fn invalid_status_enum_value_is_refused_and_writes_nothing() {
    let root = new_model();
    write(&root, "REQ-SET-001.md", REQ_MD);
    let before = read(&root, "REQ-SET-001.md");

    let (_stdout, stderr, code) = run(&root, &["set", "REQ-SET-001", "status=bogus"]);
    assert_ne!(code, 0);
    assert!(stderr.contains("not a valid status"), "{stderr}");
    assert!(stderr.contains("draft"), "should list allowed values: {stderr}");

    let after = read(&root, "REQ-SET-001.md");
    assert_eq!(before, after, "refused set must not touch the file");
}

#[test]
fn valid_status_write_is_byte_identical_elsewhere() {
    let root = new_model();
    write(&root, "REQ-SET-001.md", REQ_MD);

    let (_stdout, _stderr, code) = run(&root, &["set", "REQ-SET-001", "status=approved"]);
    assert_eq!(code, 0);

    let after = read(&root, "REQ-SET-001.md");
    assert!(after.contains("status: approved"), "{after}");
    // Every other line survives untouched -- including the quoting style of an
    // unrelated field, which a full YAML round-trip (apply_update_fields) would
    // have normalised away.
    assert!(after.contains("name: \"A requirement\""), "quoting of an untouched field must survive: {after}");
    assert!(after.contains("reqDomain: system"), "{after}");
    assert!(after.contains("Body.\n"), "{after}");
    assert_eq!(after.lines().count(), REQ_MD.lines().count(), "line count must be unchanged: {after}");
}

#[test]
fn dry_run_previews_without_writing() {
    let root = new_model();
    write(&root, "REQ-SET-001.md", REQ_MD);
    let before = read(&root, "REQ-SET-001.md");

    let (stdout, _stderr, code) = run(&root, &["set", "REQ-SET-001", "status=approved", "--dry-run"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("+status: approved"), "{stdout}");
    assert!(stdout.contains("-status: draft"), "{stdout}");

    let after = read(&root, "REQ-SET-001.md");
    assert_eq!(before, after, "--dry-run must not write");
}

#[test]
fn works_by_qualified_name_as_well_as_stable_id() {
    let root = new_model();
    write(
        &root,
        "Widgets/Widget.md",
        "---\nid: REQ-SET-002\ntype: Requirement\nname: \"Another\"\nstatus: draft\nreqDomain: system\n---\n\nBody.\n",
    );
    // by qname
    let (_out, _err, code) = run(&root, &["set", "Widgets::Widget", "status=approved", "--dry-run"]);
    assert_eq!(code, 0, "must resolve by qualified name");
    // by stable id
    let (_out, _err, code) = run(&root, &["set", "REQ-SET-002", "status=approved", "--dry-run"]);
    assert_eq!(code, 0, "must resolve by stable id");
}

#[test]
fn achieves_add_refuses_a_dangling_target() {
    let root = new_model();
    write(
        &root,
        "PI-SET-001.md",
        "---\nid: PI-SET-001\ntype: PlanningItem\nname: \"An item\"\nstatus: in_progress\nachieves: [REQ-SET-999]\n---\n\nBody.\n",
    );
    write(&root, "REQ-SET-999.md", "---\nid: REQ-SET-999\ntype: Requirement\nname: \"Exists\"\nstatus: draft\nreqDomain: system\n---\n\nBody.\n");
    let before = read(&root, "PI-SET-001.md");

    let (_out, stderr, code) = run(&root, &["set", "PI-SET-001", "achieves.add", "REQ-NOPE-NONEXISTENT"]);
    assert_ne!(code, 0);
    assert!(stderr.contains("does not resolve"), "{stderr}");

    let after = read(&root, "PI-SET-001.md");
    assert_eq!(before, after, "refused achieves.add must not touch the file");
}

#[test]
fn achieves_add_refuses_a_non_requirement_target() {
    let root = new_model();
    write(
        &root,
        "PartX.md",
        "---\nid: PART-X\ntype: PartDef\nname: PartX\n---\n\nBody.\n",
    );
    write(
        &root,
        "PI-SET-001.md",
        "---\nid: PI-SET-001\ntype: PlanningItem\nname: \"An item\"\nstatus: in_progress\nparent: PART-X\n---\n\nBody.\n",
    );
    let (_out, stderr, code) = run(&root, &["set", "PI-SET-001", "achieves.add", "PartX"]);
    assert_ne!(code, 0);
    assert!(stderr.contains("native Requirement"), "{stderr}");
}

#[test]
fn achieves_add_appends_without_disturbing_existing_order() {
    let root = new_model();
    write(&root, "REQ-A.md", "---\nid: REQ-SET-010\ntype: Requirement\nname: \"A\"\nstatus: draft\nreqDomain: system\n---\n\nBody.\n");
    write(&root, "REQ-B.md", "---\nid: REQ-SET-011\ntype: Requirement\nname: \"B\"\nstatus: draft\nreqDomain: system\n---\n\nBody.\n");
    write(
        &root,
        "PI-SET-002.md",
        "---\nid: PI-SET-002\ntype: PlanningItem\nname: \"An item\"\nstatus: in_progress\nachieves: [REQ-SET-010]\n---\n\nBody.\n",
    );
    let (_out, _err, code) = run(&root, &["set", "PI-SET-002", "achieves.add", "REQ-SET-011"]);
    assert_eq!(code, 0);
    let after = read(&root, "PI-SET-002.md");
    let a_pos = after.find("REQ-SET-010").expect("A present");
    let b_pos = after.find("REQ-SET-011").expect("B present");
    assert!(a_pos < b_pos, "existing entry must stay before the appended one: {after}");
}

#[test]
fn evidence_add_ref_must_resolve() {
    let root = new_model();
    write(
        &root,
        "PI-SET-003.md",
        "---\nid: PI-SET-003\ntype: PlanningItem\nname: \"An item\"\nstatus: in_progress\n---\n\nBody.\n",
    );
    let (_out, stderr, code) = run(&root, &["set", "PI-SET-003", "evidence.add", "ref=NOPE-NOT-REAL"]);
    assert_ne!(code, 0);
    assert!(stderr.contains("does not resolve"), "{stderr}");
}

#[test]
fn evidence_add_path_must_exist_or_be_remote() {
    let root = new_model();
    write(
        &root,
        "PI-SET-004.md",
        "---\nid: PI-SET-004\ntype: PlanningItem\nname: \"An item\"\nstatus: in_progress\n---\n\nBody.\n",
    );
    // Nonexistent local path -> refused.
    let (_out, stderr, code) = run(&root, &["set", "PI-SET-004", "evidence.add", "path=src/nope.rs"]);
    assert_ne!(code, 0);
    assert!(stderr.contains("does not exist on disk"), "{stderr}");

    // Existing local path -> accepted.
    write(&root, "src/real.rs", "// code\n");
    let (_out, _err, code) = run(&root, &["set", "PI-SET-004", "evidence.add", "path=src/real.rs", "--dry-run"]);
    assert_eq!(code, 0);

    // Remote URI -> accepted without a disk check.
    let (_out, _err, code) = run(&root, &["set", "PI-SET-004", "evidence.add", "path=https://example.com/report.html", "--dry-run"]);
    assert_eq!(code, 0);
}

/// GH #152: an evidence entry naming an already-listed target is a reported
/// no-op (like `achieves.add`), and appending a new one keeps YAML comments.
#[test]
fn evidence_add_is_idempotent_and_keeps_comments() {
    let root = new_model();
    write(&root, "REQ-A.md", "---\nid: REQ-SET-010\ntype: Requirement\nname: \"A\"\nstatus: draft\nreqDomain: system\n---\n\nBody.\n");
    write(&root, "REQ-B.md", "---\nid: REQ-SET-011\ntype: Requirement\nname: \"B\"\nstatus: draft\nreqDomain: system\n---\n\nBody.\n");
    let pi = "---\nid: PI-SET-005\ntype: PlanningItem\n# keep: header comment\nname: \"An item\"\nstatus: in_progress\nevidence:\n  - ref: REQ-SET-010  # keep: trailing comment\n---\n\nBody.\n";
    write(&root, "PI-SET-005.md", pi);

    let (stdout, _err, code) = run(&root, &["set", "PI-SET-005", "evidence.add", "ref=REQ-SET-010"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("nothing to do"), "{stdout}");
    assert_eq!(read(&root, "PI-SET-005.md"), pi, "a duplicate evidence.add must not touch the file");

    let (_out, _err, code) = run(&root, &["set", "PI-SET-005", "evidence.add", "ref=REQ-SET-011"]);
    assert_eq!(code, 0);
    let after = read(&root, "PI-SET-005.md");
    assert_eq!(
        after,
        pi.replace("comment\n---", "comment\n  - ref: REQ-SET-011\n---"),
        "only the new entry's line may be added"
    );
}

#[test]
fn planning_item_status_done_warns_on_the_w310_condition_but_still_writes() {
    let root = new_model();
    write(&root, "REQ-SET-012.md", "---\nid: REQ-SET-012\ntype: Requirement\nname: \"C\"\nstatus: draft\nreqDomain: system\n---\n\nBody.\n");
    write(
        &root,
        "PI-SET-005.md",
        "---\nid: PI-SET-005\ntype: PlanningItem\nname: \"An item\"\nstatus: in_progress\nachieves: [REQ-SET-012]\n---\n\nBody.\n",
    );
    let (_out, stderr, code) = run(&root, &["set", "PI-SET-005", "status=done"]);
    assert_eq!(code, 0, "W310 is a warning -- it must not block the write");
    assert!(stderr.contains("REQ-SET-012"), "should surface the under-verified achieves target: {stderr}");
    let after = read(&root, "PI-SET-005.md");
    assert!(after.contains("status: done"), "{after}");
}
