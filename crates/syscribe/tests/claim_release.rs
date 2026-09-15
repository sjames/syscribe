//! Regression tests for `syscribe claim`/`syscribe release` (issue #115):
//! advisory claim/ownership markers on `PlanningItem` for concurrent
//! multi-agent work.
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
        .join(format!("syscribe-claim-{}-{}-{}", std::process::id(), nanos, n))
        .join("model");
    std::fs::create_dir_all(&root).unwrap();
    write(&root, "_index.md", "---\ntype: Package\nname: ClaimTest\n---\n\nModel root.\n");
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

const PI_MD: &str = "---\nid: PI-CLAIM-001\ntype: PlanningItem\nname: \"Do the thing\"\nstatus: todo\n---\n\nBody.\n";

#[test]
fn claim_sets_claimed_by_and_claimed_at() {
    let root = new_model();
    write(&root, "PI-CLAIM-001.md", PI_MD);
    let (_out, _err, code) = run(&root, &["claim", "PI-CLAIM-001", "--by", "agent-1"]);
    assert_eq!(code, 0);
    let after = read(&root, "PI-CLAIM-001.md");
    assert!(after.contains("claimedBy: agent-1"), "{after}");
    assert!(after.contains("claimedAt:"), "{after}");
}

#[test]
fn claim_refuses_when_already_claimed_by_someone_else() {
    let root = new_model();
    write(&root, "PI-CLAIM-001.md", PI_MD);
    let (_out, _err, code) = run(&root, &["claim", "PI-CLAIM-001", "--by", "agent-1"]);
    assert_eq!(code, 0);

    let before = read(&root, "PI-CLAIM-001.md");
    let (_out, stderr, code) = run(&root, &["claim", "PI-CLAIM-001", "--by", "agent-2"]);
    assert_ne!(code, 0);
    assert!(stderr.contains("already claimed by 'agent-1'"), "{stderr}");
    let after = read(&root, "PI-CLAIM-001.md");
    assert_eq!(before, after, "a refused claim must not touch the file");
}

#[test]
fn claim_by_the_same_agent_is_allowed() {
    let root = new_model();
    write(&root, "PI-CLAIM-001.md", PI_MD);
    let (_out, _err, code) = run(&root, &["claim", "PI-CLAIM-001", "--by", "agent-1"]);
    assert_eq!(code, 0);
    let (_out, _err, code) = run(&root, &["claim", "PI-CLAIM-001", "--by", "agent-1"]);
    assert_eq!(code, 0, "re-claiming with the same agent must be allowed");
}

#[test]
fn claim_refuses_on_an_already_done_item() {
    let root = new_model();
    write(
        &root,
        "PI-CLAIM-002.md",
        "---\nid: PI-CLAIM-002\ntype: PlanningItem\nname: \"Finished\"\nstatus: done\n---\n\nBody.\n",
    );
    let (_out, stderr, code) = run(&root, &["claim", "PI-CLAIM-002", "--by", "agent-1"]);
    assert_ne!(code, 0);
    assert!(stderr.contains("nothing to claim"), "{stderr}");
    let after = read(&root, "PI-CLAIM-002.md");
    assert!(!after.contains("claimedBy"), "a refused claim must not touch the file: {after}");
}

#[test]
fn release_clears_both_fields_regardless_of_status() {
    let root = new_model();
    write(
        &root,
        "PI-CLAIM-003.md",
        "---\nid: PI-CLAIM-003\ntype: PlanningItem\nname: \"Blocked but claimed\"\nstatus: blocked\nclaimedBy: agent-1\nclaimedAt: \"2026-01-01T00:00:00Z\"\n---\n\nBody.\n",
    );
    let (_out, _err, code) = run(&root, &["release", "PI-CLAIM-003"]);
    assert_eq!(code, 0);
    let after = read(&root, "PI-CLAIM-003.md");
    assert!(!after.contains("claimedBy"), "{after}");
    assert!(!after.contains("claimedAt"), "{after}");
    assert!(after.contains("status: blocked"), "release must not touch status: {after}");
}

#[test]
fn release_on_an_unclaimed_item_is_a_no_op() {
    let root = new_model();
    write(&root, "PI-CLAIM-001.md", PI_MD);
    let before = read(&root, "PI-CLAIM-001.md");
    let (stdout, _err, code) = run(&root, &["release", "PI-CLAIM-001"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("nothing to release"), "{stdout}");
    let after = read(&root, "PI-CLAIM-001.md");
    assert_eq!(before, after);
}

#[test]
fn claimed_by_is_visible_in_show_and_list_json() {
    let root = new_model();
    write(&root, "PI-CLAIM-001.md", PI_MD);
    run(&root, &["claim", "PI-CLAIM-001", "--by", "agent-1"]);

    let (stdout, _err, _code) = run(&root, &["show", "PI-CLAIM-001"]);
    assert!(stdout.contains("**claimedBy**") && stdout.contains("agent-1"), "{stdout}");

    let (stdout, _err, _code) = run(&root, &["list", "PlanningItem", "--json"]);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
    let items = parsed.as_array().unwrap();
    let pi = items.iter().find(|v| v["id"] == "PI-CLAIM-001").expect("PI-CLAIM-001 present");
    assert_eq!(pi["claimedBy"], "agent-1");
}

#[test]
fn dry_run_previews_without_writing_for_both_commands() {
    let root = new_model();
    write(&root, "PI-CLAIM-001.md", PI_MD);
    let before = read(&root, "PI-CLAIM-001.md");
    let (stdout, _err, code) = run(&root, &["claim", "PI-CLAIM-001", "--by", "agent-1", "--dry-run"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("+claimedBy: agent-1"), "{stdout}");
    assert_eq!(read(&root, "PI-CLAIM-001.md"), before);

    run(&root, &["claim", "PI-CLAIM-001", "--by", "agent-1"]);
    let claimed = read(&root, "PI-CLAIM-001.md");
    let (stdout, _err, code) = run(&root, &["release", "PI-CLAIM-001", "--dry-run"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("-claimedBy: agent-1"), "{stdout}");
    assert_eq!(read(&root, "PI-CLAIM-001.md"), claimed, "--dry-run release must not write");
}

#[test]
fn claim_and_release_refuse_on_a_non_planning_item() {
    let root = new_model();
    write(
        &root,
        "REQ-CLAIM-001.md",
        "---\nid: REQ-CLAIM-001\ntype: Requirement\nname: \"Not a PlanningItem\"\nstatus: draft\nreqDomain: system\n---\n\nBody.\n",
    );
    let (_out, stderr, code) = run(&root, &["claim", "REQ-CLAIM-001", "--by", "agent-1"]);
    assert_ne!(code, 0);
    assert!(stderr.contains("not a PlanningItem"), "{stderr}");

    let (_out, stderr, code) = run(&root, &["release", "REQ-CLAIM-001"]);
    assert_ne!(code, 0);
    assert!(stderr.contains("not a PlanningItem"), "{stderr}");
}
