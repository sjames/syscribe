//! TC-TRS-MCPPROV-001 (GH #443/#444): MCP write tools take `by`/`reason`/`commit`/`expect_hash`, the `log`
//! tool reads git back, and `--read-only` hides exactly the mutating tools.

mod common;
use common::*;
use serde_json::json;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

fn git(root: &Path, args: &[&str]) -> String {
    let o = Command::new("git").arg("-C").arg(root).args(args).output().unwrap();
    assert!(o.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8_lossy(&o.stdout).trim().to_string()
}

/// A fixture model inside a fresh git repository, with a PlanningItem and its requirement.
fn repo() -> (PathBuf, PathBuf) {
    let model = fixture_copy();
    let root = model.parent().unwrap().to_path_buf();
    git(&root, &["init", "-q", "-b", "main"]);
    git(&root, &["config", "user.email", "human@example.com"]);
    git(&root, &["config", "user.name", "Human"]);
    git(&root, &["config", "commit.gpgsign", "false"]);
    std::fs::write(
        model.join("PI-MP-001.md"),
        "---\nid: PI-MP-001\ntype: PlanningItem\nname: Item\nstatus: todo\nachieves: [REQ-FX-001]\nquestions:\n  - id: Q1\n    text: Which port?\n    status: open\n---\n\nBody.\n",
    )
    .unwrap();
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-q", "-m", "base"]);
    (root, model)
}

#[test]
fn update_element_can_commit_with_the_actor_and_reason() {
    let (root, model) = repo();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let n = git(&root, &["rev-list", "--count", "HEAD"]);
    // Without `commit` the file changes but nothing is committed.
    let r = mcp.call_tool("update_element", json!({"ref": "PI-MP-001", "fields": {"status": "in_progress"}, "dry_run": false}));
    assert_eq!(r["written"], true, "{r}");
    assert!(r.get("commit").is_none());
    assert_eq!(git(&root, &["rev-list", "--count", "HEAD"]), n);

    let r = mcp.call_tool(
        "update_element",
        json!({"ref": "PI-MP-001", "fields": {"status": "done"}, "dry_run": false, "by": "agent-5", "reason": "tests pass", "commit": true}),
    );
    assert_eq!(r["written"], true, "{r}");
    assert!(r["commit"].as_str().is_some_and(|c| c.len() >= 7), "{r}");
    assert_eq!(git(&root, &["log", "-1", "--format=%an"]), "agent-5");
    let msg = git(&root, &["log", "-1", "--format=%B"]);
    assert!(msg.contains("Syscribe-Command: update_element update PI-MP-001") && msg.contains("Syscribe-Actor: agent-5") && msg.contains("Syscribe-Reason: tests pass"), "{msg}");
    // Both edits were to the same file; the commit holds what the file now contains relative to the last commit.
    assert_eq!(git(&root, &["show", "--name-only", "--format=", "HEAD"]), "model/PI-MP-001.md");
    assert_eq!(git(&root, &["status", "--porcelain"]), "");
}

#[test]
fn a_dry_run_never_commits_and_other_files_are_never_swept_in() {
    let (root, model) = repo();
    std::fs::write(root.join("unrelated.txt"), "pending\n").unwrap();
    git(&root, &["add", "unrelated.txt"]);
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let n = git(&root, &["rev-list", "--count", "HEAD"]);
    let r = mcp.call_tool("update_element", json!({"ref": "PI-MP-001", "fields": {"status": "done"}, "by": "agent-5", "commit": true}));
    assert_eq!(r["written"], false, "dry_run defaults to true: {r}");
    assert_eq!(git(&root, &["rev-list", "--count", "HEAD"]), n);
    let r = mcp.call_tool("update_element", json!({"ref": "PI-MP-001", "fields": {"status": "done"}, "dry_run": false, "by": "agent-5", "commit": true}));
    assert!(r["commit"].is_string(), "{r}");
    assert_eq!(git(&root, &["show", "--name-only", "--format=", "HEAD"]), "model/PI-MP-001.md");
    assert!(git(&root, &["status", "--porcelain"]).contains("A  unrelated.txt"), "the staged unrelated file stays staged");
}

#[test]
fn create_move_delete_and_question_commit_too() {
    let (root, model) = repo();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let r = mcp.call_tool(
        "create_element",
        json!({"type": "PlanningItem", "parent": "", "fields": {"id": "PI-MP-002", "name": "New", "status": "todo", "achieves": ["REQ-FX-001"]}, "doc": "x\n", "dry_run": false, "by": "agent-6", "commit": true}),
    );
    assert!(r["commit"].is_string(), "{r}");
    assert_eq!(git(&root, &["show", "--name-only", "--format=", "HEAD"]), "model/PI-MP-002.md");
    let msg = git(&root, &["log", "-1", "--format=%B"]);
    assert!(msg.contains("Syscribe-Command: create_element create PlanningItem PI-MP-002"), "{msg}");

    let r = mcp.call_tool("question", json!({"item": "PI-MP-001", "op": "answer", "id": "Q1", "text": "UART2", "by": "agent-7", "reason": "datasheet", "dry_run": false, "commit": true}));
    assert!(r["commit"].is_string(), "{r}");
    assert_eq!(git(&root, &["log", "-1", "--format=%an"]), "agent-7");
    assert!(std::fs::read_to_string(model.join("PI-MP-001.md")).unwrap().contains("answeredBy: agent-7"), "the actor is the answerer");

    let r = mcp.call_tool("delete_element", json!({"ref": "PI-MP-002", "dry_run": false, "by": "agent-6", "commit": true}));
    assert!(r["commit"].is_string(), "{r}");
    assert_eq!(git(&root, &["show", "--name-status", "--format=", "HEAD"]), "D\tmodel/PI-MP-002.md");

    let r = mcp.call_tool("move_element", json!({"ref": "PI-MP-001", "dest": "PI-MP-009", "dry_run": false, "by": "agent-8", "commit": true}));
    assert!(r["commit"].is_string() || r["written"] == false, "{r}");
}

/// The files to commit come from comparing content before and after the write, not from re-reading the diff
/// text: a file whose only change is a trailing newline has no diff hunk, yet it did change.
#[test]
fn a_change_with_no_diff_hunk_is_still_committed() {
    let (root, model) = repo();
    let path = model.join("PI-MP-001.md");
    std::fs::write(&path, std::fs::read_to_string(&path).unwrap().trim_end_matches('\n').to_string()).unwrap();
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-q", "-m", "no trailing newline"]);
    let before = std::fs::read(&path).unwrap();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    // Re-write the body with a trailing newline: line-for-line identical, byte-for-byte different.
    let r = mcp.call_tool("update_element", json!({"ref": "PI-MP-001", "doc": "\nBody.\n", "dry_run": false, "by": "agent-5", "commit": true}));
    let after = std::fs::read(&path).unwrap();
    assert_ne!(before, after, "the precondition: the write changes the bytes of the file ({r})");
    assert_eq!(r["written"], true, "{r}");
    assert!(r["commit"].is_string(), "a changed file must be committed even with no diff hunk: {r}");
    assert_eq!(git(&root, &["status", "--porcelain"]), "", "nothing left uncommitted");
}

#[test]
fn expect_hash_refuses_a_stale_write_over_mcp() {
    let (root, model) = repo();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let e = mcp.call_tool("get_element", json!({"ref": "PI-MP-001", "detail": true}));
    let h = e["contentHash"].as_str().expect("get_element detail carries contentHash").to_string();
    assert!(h.starts_with("blake3:"));
    // Someone edits the file on disk after the read.
    let path = model.join("PI-MP-001.md");
    std::fs::write(&path, std::fs::read_to_string(&path).unwrap().replace("Body.", "Body, edited elsewhere.")).unwrap();
    let before = std::fs::read_to_string(&path).unwrap();
    for (tool, args) in [
        ("update_element", json!({"ref": "PI-MP-001", "fields": {"status": "done"}, "dry_run": false, "expect_hash": h})),
        ("question", json!({"item": "PI-MP-001", "op": "defer", "id": "Q1", "dry_run": false, "expect_hash": h})),
        ("delete_element", json!({"ref": "PI-MP-001", "force": true, "dry_run": false, "expect_hash": h})),
        ("move_element", json!({"ref": "PI-MP-001", "dest": "PI-MP-077", "dry_run": false, "expect_hash": h})),
    ] {
        let r = mcp.call_tool_raw(tool, args);
        assert_eq!(r["isError"], true, "{tool}: {r}");
        assert!(serde_json::to_string(&r).unwrap().contains("has changed since it was read"), "{tool}: {r}");
    }
    assert_eq!(std::fs::read_to_string(&path).unwrap(), before, "a stale write changes nothing");
    let _ = root;
}

#[test]
fn a_bad_actor_is_refused() {
    let (_root, model) = repo();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let r = mcp.call_tool_raw("update_element", json!({"ref": "PI-MP-001", "fields": {"status": "done"}, "dry_run": false, "by": "not valid!"}));
    assert_eq!(r["isError"], true, "{r}");
}

#[test]
fn the_log_tool_reads_git_back() {
    let (root, model) = repo();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    mcp.call_tool("update_element", json!({"ref": "PI-MP-001", "fields": {"status": "in_progress"}, "dry_run": false, "by": "agent-9", "reason": "starting", "commit": true}));
    let l = mcp.call_tool("log", json!({"element": "PI-MP-001"}));
    assert_eq!(l["entries"][0]["actor"], "agent-9");
    assert_eq!(l["entries"][0]["reason"], "starting");
    assert_eq!(l["entries"][0]["change"], "status todo → in_progress");
    assert_eq!(mcp.call_tool("log", json!({"unattributed": true}))["entries"].as_array().unwrap().len(), 1, "only the base commit");
    assert_eq!(mcp.call_tool("log", json!({"by": "agent-9"}))["entries"].as_array().unwrap().len(), 1);
    assert_eq!(mcp.call_tool_raw("log", json!({"element": "NOPE-1"}))["isError"], true);
    let _ = root;
}

/// `--read-only` hides exactly the tools that can write: every tool not annotated read-only disappears, and
/// no read-only tool does.
#[test]
fn read_only_hides_exactly_the_mutating_tools() {
    let model = fixture_copy();
    let mut full = Mcp::start(&model);
    full.initialize();
    let all = full.tools_list();
    let all = all["tools"].as_array().unwrap();
    let mutating: BTreeSet<String> = all
        .iter()
        .filter(|t| t["annotations"]["readOnlyHint"].as_bool() != Some(true))
        .map(|t| t["name"].as_str().unwrap().to_string())
        .collect();
    assert!(mutating.contains("update_element") && mutating.contains("question") && mutating.contains("ingest_results"), "{mutating:?}");

    let mut ro = Mcp::start_with_args(&model, &["--read-only"]);
    ro.initialize();
    let visible = ro.tools_list();
    let visible: BTreeSet<String> = visible["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap().to_string()).collect();
    let every: BTreeSet<String> = all.iter().map(|t| t["name"].as_str().unwrap().to_string()).collect();
    let hidden: BTreeSet<String> = every.difference(&visible).cloned().collect();
    assert_eq!(hidden, mutating, "the hidden tools are exactly the ones not annotated read-only");
    for t in ro.tools_list()["tools"].as_array().unwrap() {
        assert_eq!(t["annotations"]["readOnlyHint"], true, "{} is visible under --read-only so must be read-only", t["name"]);
    }
    // And a hidden tool cannot be called anyway.
    assert_eq!(ro.call_tool_raw("update_element", json!({"ref": "REQ-FX-001", "fields": {"status": "approved"}, "dry_run": false}))["isError"], true);
}
