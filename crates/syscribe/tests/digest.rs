//! Integration tests for `syscribe digest` and the MCP `digest` tool
//! (TC-TRS-OUT-022 → REQ-TRS-OUT-022).

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

fn fixture_model() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/model")
}

fn run_digest(extra: &[&str]) -> (String, String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m").arg(fixture_model())
        .arg("digest")
        .args(extra)
        .output()
        .expect("spawn digest");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

fn digest_json(extra: &[&str]) -> Value {
    let mut args = vec!["--json"];
    args.extend_from_slice(extra);
    let (stdout, stderr, code) = run_digest(&args);
    assert_eq!(code, 0, "digest --json exits 0 (stderr: {stderr})");
    serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("digest --json valid: {e}\n{stdout}"))
}

// ---- Scenario: default output is one compact NDJSON row per requirement -----

#[test]
fn default_output_is_ndjson_rows() {
    let (stdout, stderr, code) = run_digest(&[]);
    assert_eq!(code, 0, "digest exits 0 (stderr: {stderr})");
    let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines.len(), 6, "6 requirements → 6 NDJSON rows");
    for line in lines {
        let row: Value = serde_json::from_str(line).expect("each line is valid JSON");
        for k in ["id", "status", "reqDomain", "text", "verified"] {
            assert!(row.get(k).is_some(), "row carries '{k}': {line}");
        }
        let text = row["text"].as_str().unwrap();
        assert!(!text.contains('\n'), "text is a single line");
        assert!(text.chars().count() <= 201, "text is length-bounded");
    }
}

// ---- Scenario: --json emits a paged document with a pre-paging total --------

#[test]
fn json_document_pages_with_full_total() {
    let d = digest_json(&["--limit", "3", "--offset", "2"]);
    assert_eq!(d["total"].as_u64(), Some(6), "total is the full count, not the page");
    assert_eq!(d["offset"].as_u64(), Some(2));
    assert_eq!(d["rows"].as_array().unwrap().len(), 3, "at most 3 rows");
}

// ---- Scenario: scoping filters restrict the rows and the total --------------

#[test]
fn status_filter_restricts_rows_and_total() {
    let d = digest_json(&["--status", "approved"]);
    assert_eq!(d["total"].as_u64(), Some(3), "3 approved requirements");
    for row in d["rows"].as_array().unwrap() {
        assert_eq!(row["status"].as_str(), Some("approved"));
    }
}

// ---- Scenario: --config projects the rows onto a variant --------------------

#[test]
fn config_projects_the_rows() {
    // CONF-FX-001 deselects Sat → REQ-FXSAT-001 projected out (6 → 5).
    let d = digest_json(&["--config", "CONF-FX-001"]);
    assert_eq!(d["total"].as_u64(), Some(5), "one requirement projected out");
    let ids: Vec<&str> = d["rows"].as_array().unwrap().iter().map(|r| r["id"].as_str().unwrap()).collect();
    assert!(!ids.contains(&"REQ-FXSAT-001"), "gated requirement absent");
}

#[test]
fn unresolvable_config_is_usage_error() {
    let (_o, _e, code) = run_digest(&["--config", "bogus", "--json"]);
    assert_eq!(code, 1, "unresolvable --config exits 1");
}

// ---- Scenario: the MCP digest tool matches the CLI --------------------------

#[test]
fn mcp_digest_tool_matches_cli() {
    use common::Mcp;
    let model = common::fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();

    let tools = mcp.tools_list();
    let tool = tools
        .get("tools").and_then(|t| t.as_array())
        .and_then(|a| a.iter().find(|t| t.get("name").and_then(|n| n.as_str()) == Some("digest")))
        .expect("digest tool advertised");
    assert_eq!(
        tool.pointer("/annotations/readOnlyHint").and_then(|v| v.as_bool()),
        Some(true),
        "digest tool readOnlyHint=true"
    );

    let tool_doc = mcp.call_tool("digest", serde_json::json!({}));
    let cli = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m").arg(&model)
        .arg("digest").arg("--json")
        .output()
        .expect("spawn digest");
    let cli_doc: Value = serde_json::from_slice(&cli.stdout).expect("cli json");
    assert_eq!(tool_doc, cli_doc, "MCP digest tool == digest --json");
}
