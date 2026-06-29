//! Integration tests for the run_report passthrough (Bundle F).
//! Realises TC-TRS-MCP-034.

mod common;
use common::*;
use serde_json::json;

#[test]
fn allowlisted_report_runs() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("run_report", json!({"command": "matrix"}));
    assert_eq!(res.get("exitCode").and_then(|c| c.as_i64()), Some(0), "matrix exits 0: {res}");
    let out = res.get("output").and_then(|o| o.as_str()).unwrap_or("");
    assert!(!out.is_empty(), "report output is non-empty");
}

#[test]
fn disallowed_command_is_refused() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    // `move` is a write command — not on the allowlist.
    let res = mcp.call_tool_raw("run_report", json!({"command": "move", "args": ["Parts::Base", "Parts::X"]}));
    assert_eq!(res.get("isError").and_then(|e| e.as_bool()), Some(true), "disallowed command refused: {res}");
    assert!(mcp.is_alive(), "server still serving after a refusal");
}

#[test]
fn model_root_redirect_is_refused() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    // Attempt to point the command at another model root via injected -m.
    let res = mcp.call_tool_raw("run_report", json!({"command": "matrix", "args": ["-m", "/etc"]}));
    assert_eq!(res.get("isError").and_then(|e| e.as_bool()), Some(true), "model redirection refused: {res}");
}
