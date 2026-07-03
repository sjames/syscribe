//! Integration tests for `syscribe summarize` and the MCP `summarize` tool
//! (TC-TRS-OUT-023 → REQ-TRS-OUT-023). Uses a fixture COPY because summarize
//! writes a cache into the model tree.

mod common;

use std::path::Path;
use std::process::Command;

use serde_json::Value;

fn run_summarize(model: &Path, extra: &[&str]) -> (String, String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m").arg(model)
        .arg("summarize").args(extra)
        .output().expect("spawn summarize");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

fn summarize_json(model: &Path, extra: &[&str]) -> Value {
    let mut args = vec!["--json"];
    args.extend_from_slice(extra);
    let (stdout, stderr, code) = run_summarize(model, &args);
    assert_eq!(code, 0, "summarize --json exits 0 (stderr: {stderr})");
    serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("summarize --json valid: {e}\n{stdout}"))
}

#[test]
fn digest_is_a_nested_per_package_rollup() {
    let model = common::fixture_copy();
    let d = summarize_json(&model, &[]);
    assert_eq!(d["qname"].as_str(), Some("(root)"));
    assert_eq!(d["count"].as_u64(), Some(6), "6 requirements in scope");
    for k in ["qname", "count", "statusSplit", "terms", "representative", "children"] {
        assert!(d.get(k).is_some(), "root node carries '{k}'");
    }
    let children = d["children"].as_array().unwrap();
    let child_names: Vec<&str> = children.iter().map(|c| c["qname"].as_str().unwrap()).collect();
    assert!(child_names.contains(&"Requirements"), "child package Requirements present: {child_names:?}");
    // representative entries carry id + text
    let reqs_node = children.iter().find(|c| c["qname"].as_str() == Some("Requirements")).unwrap();
    let rep = &reqs_node["representative"].as_array().unwrap()[0];
    assert!(rep["id"].is_string() && rep["text"].is_string(), "representative has id+text");
}

#[test]
fn about_terms_are_content_words() {
    let model = common::fixture_copy();
    let d = summarize_json(&model, &[]);
    let terms: Vec<&str> = d["terms"].as_array().unwrap().iter().map(|t| t.as_str().unwrap()).collect();
    for sw in ["the", "shall", "and", "is"] {
        assert!(!terms.contains(&sw), "stopword '{sw}' present in summarize terms: {terms:?}");
    }
}

#[test]
fn output_is_deterministic_and_cached() {
    let model = common::fixture_copy();
    let (a, _, _) = run_summarize(&model, &[]);
    let cache = model.join(".syscribe").join("cache").join("summaries.json");
    assert!(cache.exists(), "cache file written after first run");
    let (b, _, _) = run_summarize(&model, &[]);
    assert_eq!(a, b, "second run (served from cache) is identical");
}

#[test]
fn scope_and_config_restrict_the_digest() {
    let model = common::fixture_copy();
    // CONF-FX-001 gates out REQ-FXSAT-001 → root count 5.
    let d = summarize_json(&model, &["--config", "CONF-FX-001"]);
    assert_eq!(d["count"].as_u64(), Some(5), "gated requirement excluded");
    let (_o, _e, code) = run_summarize(&model, &["--scope", "NoSuchPackage"]);
    assert_eq!(code, 1, "unresolvable --scope exits 1");
}

#[test]
fn mcp_summarize_tool_matches_cli() {
    use common::Mcp;
    let model = common::fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();

    let tools = mcp.tools_list();
    let tool = tools.get("tools").and_then(|t| t.as_array())
        .and_then(|a| a.iter().find(|t| t.get("name").and_then(|n| n.as_str()) == Some("summarize")))
        .expect("summarize tool advertised");
    assert_eq!(tool.pointer("/annotations/readOnlyHint").and_then(|v| v.as_bool()), Some(true));

    let tool_doc = mcp.call_tool("summarize", serde_json::json!({}));
    let cli = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m").arg(&model).arg("summarize").arg("--json")
        .output().expect("spawn summarize");
    let cli_doc: Value = serde_json::from_slice(&cli.stdout).expect("cli json");
    assert_eq!(tool_doc, cli_doc, "MCP summarize == summarize --json");
}
