//! Integration tests for the MCP suspect-link tools (`suspect_list`,
//! `suspect_accept`). Realises TC-TRS-MCP-046 (verifies REQ-TRS-MCP-045).
//!
//! Drives a real `syscribe mcp` subprocess over stdio against a copy of the shared
//! fixture (its `TC-FX-001` verifies `REQ-FX-001` — a baselineable link).

mod common;
use common::*;
use serde_json::{json, Value};

/// A hash that cannot match any real projection → a guaranteed-stale baseline.
const STALE: &str = "blake3:0000000000000000000000000000000000000000000000000000000000000000";

/// Inject a `traceBaselines` entry into a fixture source file's frontmatter,
/// just before the closing `---`, so the link starts out suspect.
fn inject_baseline(model: &std::path::Path, rel: &str, target: &str, hash: &str) {
    let path = model.join(rel);
    let content = std::fs::read_to_string(&path).unwrap();
    // Insert before the second `---` (the frontmatter terminator).
    let close = content[3..].find("\n---").expect("frontmatter close") + 3;
    let insert = format!("\ntraceBaselines:\n  {target}: \"{hash}\"");
    let patched = format!("{}{}{}", &content[..close], insert, &content[close..]);
    std::fs::write(&path, patched).unwrap();
}

fn arr<'a>(v: &'a Value, key: &str) -> &'a Vec<Value> {
    v.get(key).and_then(|x| x.as_array()).unwrap_or_else(|| panic!("missing array `{key}` in {v}"))
}

/// Tool names from a `tools/list` result.
fn tool_names(list: &Value) -> Vec<String> {
    list.get("tools")
        .and_then(|t| t.as_array())
        .map(|a| a.iter().filter_map(|t| t.get("name").and_then(|n| n.as_str()).map(String::from)).collect())
        .unwrap_or_default()
}

// ---- suspect_list -----------------------------------------------------------

#[test]
fn suspect_list_reports_unbaselined_link() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();

    let res = mcp.call_tool("suspect_list", json!({}));
    let unbaselined = arr(&res, "unbaselined");
    let hit = unbaselined.iter().any(|l| {
        l.get("source").and_then(|s| s.as_str()) == Some("TC-FX-001")
            && l.get("target").and_then(|t| t.as_str()) == Some("REQ-FX-001")
            && l.get("kind").and_then(|k| k.as_str()) == Some("verifies")
    });
    assert!(hit, "unbaselined verifies link is reported with source, target, kind: {res}");
}

#[test]
fn suspect_list_reports_suspect_link() {
    let model = fixture_copy();
    inject_baseline(&model, "Verification/TC-FX-001.md", "REQ-FX-001", STALE);
    let mut mcp = Mcp::start(&model);
    mcp.initialize();

    let res = mcp.call_tool("suspect_list", json!({}));
    let suspect = arr(&res, "suspect");
    let hit = suspect.iter().any(|l| l.get("source").and_then(|s| s.as_str()) == Some("TC-FX-001"));
    assert!(hit, "the stale link is reported as suspect: {res}");
}

// ---- suspect_accept: write guard --------------------------------------------

#[test]
fn suspect_accept_dry_run_does_not_write() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let before = dir_hash(&model);

    let res = mcp.call_tool("suspect_accept", json!({"source": "TC-FX-001", "target": "REQ-FX-001"}));
    assert_eq!(res.get("written").and_then(|w| w.as_bool()), Some(false), "dry-run does not write: {res}");
    assert!(res.get("validationDelta").is_some(), "dry-run returns a validation delta: {res}");
    assert!(res.get("diff").and_then(|d| d.as_str()).is_some_and(|d| !d.is_empty()), "dry-run returns a diff: {res}");
    assert_eq!(before, dir_hash(&model), "dry-run leaves disk byte-for-byte unchanged");
}

#[test]
fn suspect_accept_commit_baselines_and_resolves_w090() {
    let model = fixture_copy();
    // Start suspect: a stale baseline exists → W090 fires.
    inject_baseline(&model, "Verification/TC-FX-001.md", "REQ-FX-001", STALE);
    let mut mcp = Mcp::start(&model);
    mcp.initialize();

    let res = mcp.call_tool(
        "suspect_accept",
        json!({"source": "TC-FX-001", "target": "REQ-FX-001", "dry_run": false}),
    );
    assert_eq!(res.get("written").and_then(|w| w.as_bool()), Some(true), "commit writes: {res}");

    // The stale hash is replaced by a real one.
    let src = std::fs::read_to_string(model.join("Verification/TC-FX-001.md")).unwrap();
    assert!(src.contains("blake3:") && !src.contains(STALE), "stale baseline replaced by current hash:\n{src}");

    // Clearing the suspect link resolves the W090 → it appears in resolvedWarnings.
    let resolved = arr(&res["validationDelta"], "resolvedWarnings");
    let has_w090 = resolved.iter().any(|f| f.get("code").and_then(|c| c.as_str()) == Some("W090"));
    assert!(has_w090, "the cleared suspect link's W090 is in resolvedWarnings: {res}");
}

#[test]
fn suspect_accept_nonreferenced_target_is_refused() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let before = dir_hash(&model);

    // REQ-FX-003 exists but is not referenced by any link on TC-FX-001.
    let res = mcp.call_tool_raw(
        "suspect_accept",
        json!({"source": "TC-FX-001", "target": "REQ-FX-003", "dry_run": false}),
    );
    assert_eq!(res.get("isError").and_then(|e| e.as_bool()), Some(true), "non-referenced target errors: {res}");
    assert_eq!(before, dir_hash(&model), "a refused accept writes nothing");
}

#[test]
fn read_only_server_refuses_suspect_accept_but_allows_list() {
    let model = fixture_copy();
    let mut mcp = Mcp::start_with_args(&model, &["--read-only"]);
    mcp.initialize();

    // suspect_accept is a write tool → hidden and refused under --read-only.
    let names = tool_names(&mcp.tools_list());
    assert!(!names.contains(&"suspect_accept".to_string()), "suspect_accept hidden in --read-only: {names:?}");
    assert!(names.contains(&"suspect_list".to_string()), "suspect_list still available in --read-only");

    let before = dir_hash(&model);
    let res = mcp.call_tool_raw(
        "suspect_accept",
        json!({"source": "TC-FX-001", "target": "REQ-FX-001", "dry_run": false}),
    );
    assert_eq!(res.get("isError").and_then(|e| e.as_bool()), Some(true), "write refused in read-only: {res}");
    assert_eq!(before, dir_hash(&model), "read-only server writes nothing");
}
