//! Integration tests for `syscribe search-text` and the MCP `search_text` tool
//! (TC-TRS-SEARCH-001 → REQ-TRS-SEARCH-001).

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

fn fixture_model() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/model")
}

fn run_search(extra: &[&str]) -> (String, String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m").arg(fixture_model())
        .arg("search-text")
        .args(extra)
        .output()
        .expect("spawn search-text");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

fn search_json(extra: &[&str]) -> Value {
    let mut args: Vec<&str> = Vec::new();
    args.extend_from_slice(extra);
    args.push("--json");
    let (stdout, stderr, code) = run_search(&args);
    assert_eq!(code, 0, "search-text --json exits 0 (stderr: {stderr})");
    serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("search-text --json valid: {e}\n{stdout}"))
}

// ---- Scenario: the most relevant element ranks first ------------------------

#[test]
fn unique_term_returns_its_element_first() {
    // "satcom" occurs only in REQ-FXSAT-001's body.
    let d = search_json(&["satcom"]);
    assert_eq!(d["total"].as_u64(), Some(1), "one document matches 'satcom'");
    let first = &d["results"][0];
    assert_eq!(first["id"].as_str(), Some("REQ-FXSAT-001"), "the matching requirement ranks first");
}

#[test]
fn results_ordered_by_descending_score() {
    // "fixture" appears in several requirement bodies.
    let d = search_json(&["fixture"]);
    let scores: Vec<f64> = d["results"].as_array().unwrap().iter().map(|r| r["score"].as_f64().unwrap()).collect();
    assert!(scores.len() >= 2, "several documents match 'fixture'");
    for w in scores.windows(2) {
        assert!(w[0] >= w[1], "scores are non-increasing: {scores:?}");
    }
}

// ---- Scenario: each result carries a marked snippet -------------------------

#[test]
fn result_carries_marked_snippet() {
    let d = search_json(&["satcom"]);
    let r = &d["results"][0];
    for k in ["qname", "type", "score", "snippet"] {
        assert!(r.get(k).is_some(), "result carries '{k}'");
    }
    let snippet = r["snippet"].as_str().unwrap();
    assert!(snippet.to_lowercase().contains("**satcom**"), "snippet marks the term: {snippet}");
}

// ---- Scenario: --type restricts the searched set ----------------------------

#[test]
fn type_filter_restricts_results() {
    let d = search_json(&["fixture", "--type", "Requirement"]);
    for r in d["results"].as_array().unwrap() {
        assert_eq!(r["type"].as_str(), Some("Requirement"), "every result is a Requirement");
    }
    assert!(d["total"].as_u64().unwrap() >= 1, "at least one Requirement matches");
}

// ---- Scenario: an empty query is a usage error ------------------------------

#[test]
fn empty_query_is_usage_error() {
    let (_o, _e, code) = run_search(&["", "--json"]);
    assert_eq!(code, 1, "empty query exits 1");
}

// ---- Scenario: --config searches only the variant ---------------------------

#[test]
fn config_scopes_the_search() {
    // CONF-FX-001 deselects Sat → REQ-FXSAT-001 (the only 'satcom' doc) projected out.
    let d = search_json(&["satcom", "--config", "CONF-FX-001"]);
    assert_eq!(d["total"].as_u64(), Some(0), "gated requirement is not searched");
}

#[test]
fn unresolvable_config_is_usage_error() {
    let (_o, _e, code) = run_search(&["satcom", "--config", "bogus", "--json"]);
    assert_eq!(code, 1, "unresolvable --config exits 1");
}

// ---- Scenario: the MCP search_text tool matches the CLI ---------------------

#[test]
fn mcp_search_text_tool_matches_cli() {
    use common::Mcp;
    let model = common::fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();

    let tools = mcp.tools_list();
    let tool = tools
        .get("tools").and_then(|t| t.as_array())
        .and_then(|a| a.iter().find(|t| t.get("name").and_then(|n| n.as_str()) == Some("search_text")))
        .expect("search_text tool advertised");
    assert_eq!(
        tool.pointer("/annotations/readOnlyHint").and_then(|v| v.as_bool()),
        Some(true),
        "search_text tool readOnlyHint=true"
    );

    let tool_doc = mcp.call_tool("search_text", serde_json::json!({ "query": "fixture" }));
    let cli = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m").arg(&model)
        .arg("search-text").arg("fixture").arg("--json")
        .output()
        .expect("spawn search-text");
    let cli_doc: Value = serde_json::from_slice(&cli.stdout).expect("cli json");
    assert_eq!(tool_doc, cli_doc, "MCP search_text tool == search-text --json");
}
