//! Integration tests for the MCP baseline read tools (`baseline_list`,
//! `baseline_diff`, `baseline_verify`). Realises TC-TRS-MCP-047 (verifies
//! REQ-TRS-MCP-046).
//!
//! Builds a git-backed model, seals two baselines via the `syscribe` CLI (one
//! then drifted), and drives a real `syscribe mcp` subprocess against it.

mod common;
use common::*;
use serde_json::{json, Value};

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn git(dir: &Path, args: &[&str]) {
    assert!(Command::new("git").arg("-C").arg(dir).args(args).output().unwrap().status.success(), "git {args:?}");
}
fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}
fn cli(model: &Path, args: &[&str]) {
    let ok = Command::new(env!("CARGO_BIN_EXE_syscribe")).arg("-m").arg(model).args(args).output().unwrap();
    // create/commit may print to stderr; we only require it to have run.
    let _ = ok;
}
fn req(root: &Path, id: &str, body: &str) {
    write(root, &format!("Requirements/{id}.md"),
        &format!("---\ntype: Requirement\nid: {id}\nname: \"{id}\"\nstatus: approved\nreqDomain: software\nreqClass: system\n---\n\n{body}\n"));
}
fn tool_names(list: &Value) -> Vec<String> {
    list.get("tools").and_then(|t| t.as_array())
        .map(|a| a.iter().filter_map(|t| t.get("name").and_then(|n| n.as_str()).map(String::from)).collect())
        .unwrap_or_default()
}

/// Git model with two sealed baselines: BL-2026-06 (then drifted by a change to
/// REQ-MOD-001) and BL-2026-07 (sealed after the change → intact).
fn model_with_baselines() -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let repo = std::env::temp_dir().join(format!("syscribe-mcpbl-{}-{}-{}", std::process::id(), nanos, n));
    let model = repo.join("model");
    std::fs::create_dir_all(model.join("Requirements")).unwrap();
    write(&model, "_index.md", "---\ntype: Package\nname: M\n---\n\nroot\n");
    req(&model, "REQ-MOD-001", "Original body one.");
    req(&model, "REQ-MOD-002", "Original body two.");
    git(&repo, &["init", "-q"]);
    git(&repo, &["config", "user.email", "t@t"]);
    git(&repo, &["config", "user.name", "t"]);
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-qm", "init"]);

    cli(&model, &["baseline", "create", "--tag", "REL-2026-06", "--approver", "J. Roe"]);
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-qm", "bl06"]);
    // Drift BL-2026-06 by changing an in-scope element, then seal BL-2026-07.
    req(&model, "REQ-MOD-001", "CHANGED body one.");
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-qm", "change"]);
    cli(&model, &["baseline", "create", "--tag", "REL-2026-07", "--approver", "J. Roe"]);
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-qm", "bl07"]);
    model
}

fn arr<'a>(v: &'a Value, key: &str) -> &'a Vec<Value> {
    v.get(key).and_then(|x| x.as_array()).unwrap_or_else(|| panic!("missing `{key}` in {v}"))
}

#[test]
fn baseline_list_inventories_sealed_baselines() {
    let model = model_with_baselines();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("baseline_list", json!({}));
    let bs = arr(&res, "baselines");
    let ids: Vec<&str> = bs.iter().filter_map(|b| b.get("id").and_then(|i| i.as_str())).collect();
    assert!(ids.contains(&"BL-2026-06") && ids.contains(&"BL-2026-07"), "both baselines listed: {res}");
    assert!(bs.iter().all(|b| b.get("aggregateHash").is_some()), "each carries an aggregateHash: {res}");
}

#[test]
fn baseline_diff_reports_the_changed_element() {
    let model = model_with_baselines();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("baseline_diff", json!({"from": "BL-2026-06", "to": "BL-2026-07"}));
    let changed = arr(&res, "changed");
    let keys: Vec<String> = changed.iter().filter_map(|e| e.get("id").and_then(|i| i.as_str()).map(String::from)).collect();
    assert!(keys.iter().any(|k| k == "REQ-MOD-001"), "REQ-MOD-001 reported changed: {res}");
    assert_eq!(res.get("aggregateIdentical").and_then(|b| b.as_bool()), Some(false), "aggregates differ");
}

#[test]
fn baseline_verify_reports_integrity() {
    let model = model_with_baselines();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("baseline_verify", json!({}));
    let results = arr(&res, "results");
    let get = |id: &str| results.iter().find(|r| r.get("id").and_then(|i| i.as_str()) == Some(id)).cloned().unwrap();
    assert_eq!(get("BL-2026-07").get("passed").and_then(|p| p.as_bool()), Some(true), "intact baseline passes: {res}");
    assert_eq!(get("BL-2026-06").get("passed").and_then(|p| p.as_bool()), Some(false), "drifted baseline fails: {res}");
    assert_eq!(res.get("passed").and_then(|p| p.as_bool()), Some(false), "overall fails when any fails");
}

#[test]
fn no_baseline_create_write_tool_is_exposed() {
    let model = model_with_baselines();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let names = tool_names(&mcp.tools_list());
    assert!(!names.contains(&"baseline_create".to_string()), "no baseline_create tool: {names:?}");
    for r in ["baseline_list", "baseline_diff", "baseline_verify"] {
        assert!(names.contains(&r.to_string()), "{r} is exposed");
    }
}

#[test]
fn baseline_read_tools_are_read_only() {
    let model = model_with_baselines();
    let before = dir_hash(&model);
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    mcp.call_tool("baseline_list", json!({}));
    mcp.call_tool("baseline_diff", json!({"from": "BL-2026-06", "to": "BL-2026-07"}));
    mcp.call_tool("baseline_verify", json!({}));
    assert_eq!(before, dir_hash(&model), "baseline read tools mutate no model file");
}
