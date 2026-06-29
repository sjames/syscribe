//! Integration tests for the Retrieval bundle.
//! Realises TC-TRS-MCP-027 (filtered search) and TC-TRS-MCP-028 (config resource).

mod common;
use common::*;
use serde_json::json;

fn result_list(v: &serde_json::Value) -> Vec<serde_json::Value> {
    v.get("results").and_then(|r| r.as_array()).cloned().unwrap_or_default()
}

// ---- TC-TRS-MCP-027: filtered / full-text search ----------------------------

#[test]
fn search_type_filter_narrows_results() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("search", json!({"query": "fixture", "type": "Requirement"}));
    let results = result_list(&res);
    assert!(!results.is_empty(), "type-filtered search returns results");
    assert!(
        results.iter().all(|r| r.get("type").and_then(|t| t.as_str()) == Some("Requirement")),
        "every result is a Requirement; got {results:?}"
    );
    assert!(
        results.iter().any(|r| r.get("id").and_then(|i| i.as_str()) == Some("REQ-FX-001")),
        "REQ-FX-001 present"
    );
}

#[test]
fn search_matches_body_text() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    // "retrieve" appears only in REQ-FX-001's documentation body, not its name/id/qname.
    let res = mcp.call_tool("search", json!({"query": "retrieve"}));
    let results = result_list(&res);
    assert!(
        results.iter().any(|r| r.get("id").and_then(|i| i.as_str()) == Some("REQ-FX-001")),
        "body-text match finds REQ-FX-001; got {results:?}"
    );
}

#[test]
fn search_where_predicate_filters() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    // Parts::Base carries `customKey: keepme` in its frontmatter.
    let res = mcp.call_tool("search", json!({"where": "custom.customKey=keepme"}));
    let results = result_list(&res);
    assert!(
        results.iter().any(|r| r.get("qname").and_then(|q| q.as_str()).is_some_and(|q| q.contains("Parts::Base"))),
        "where-predicate selects the element with the custom field; got {results:?}"
    );
}

// ---- TC-TRS-MCP-028: config resource ----------------------------------------

#[test]
fn config_resource_is_readable() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let read = mcp.resources_read("syscribe://config");
    let text = read
        .get("contents")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .and_then(|i| i.get("text"))
        .and_then(|t| t.as_str())
        .expect("config resource returns text");
    assert!(text.contains("ids.prefixes") || text.contains("FXREQ"), "config text reflects the project config; got {text}");
}
