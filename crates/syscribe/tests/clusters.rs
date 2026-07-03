//! Integration tests for `syscribe clusters` and the MCP `clusters` tool
//! (TC-TRS-SEARCH-003 → REQ-TRS-SEARCH-003).

mod common;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

fn fixture_model() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/model")
}

fn run_clusters(extra: &[&str]) -> (String, String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m").arg(fixture_model())
        .arg("clusters").args(extra)
        .output().expect("spawn clusters");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

fn clusters_json(extra: &[&str]) -> Value {
    let mut args = vec!["--json"];
    args.extend_from_slice(extra);
    let (stdout, stderr, code) = run_clusters(&args);
    assert_eq!(code, 0, "clusters --json exits 0 (stderr: {stderr})");
    serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("clusters --json valid: {e}\n{stdout}"))
}

fn all_members(d: &Value) -> Vec<String> {
    d["clusters"].as_array().unwrap().iter()
        .flat_map(|c| c["members"].as_array().unwrap().iter().map(|m| m.as_str().unwrap().to_string()))
        .collect()
}

#[test]
fn k_clusters_partition_the_elements() {
    let d = clusters_json(&["--k", "2"]);
    assert_eq!(d["k"].as_u64(), Some(2), "k is 2");
    let clusters = d["clusters"].as_array().unwrap();
    assert_eq!(clusters.len(), 2, "2 clusters");
    let members = all_members(&d);
    // every element in exactly one cluster (no duplicates), 6 requirements total
    let uniq: HashSet<&String> = members.iter().collect();
    assert_eq!(uniq.len(), members.len(), "no element in two clusters");
    assert_eq!(members.len(), 6, "all 6 requirements clustered");
    let sizes: u64 = clusters.iter().map(|c| c["size"].as_u64().unwrap()).sum();
    assert_eq!(sizes, 6, "sizes sum to the element count");
    for c in clusters {
        assert!(c["label"].is_array(), "cluster has a term label");
        assert!(c["members"].is_array(), "cluster has members");
    }
}

#[test]
fn clustering_is_deterministic() {
    let a = clusters_json(&["--k", "2"]);
    let b = clusters_json(&["--k", "2"]);
    assert_eq!(a, b, "two runs produce identical clusters");
}

#[test]
fn cosine_separates_distinctive_vocabulary() {
    // REQ-FXSAT-001 (satellite/satcom) shares no vocabulary with the MCP-retrieval
    // requirement REQ-FX-001, so they must not land in the same cluster.
    let d = clusters_json(&["--k", "2"]);
    for c in d["clusters"].as_array().unwrap() {
        let members: Vec<&str> = c["members"].as_array().unwrap().iter().map(|m| m.as_str().unwrap()).collect();
        let has_sat = members.contains(&"REQ-FXSAT-001");
        let has_fx1 = members.contains(&"REQ-FX-001");
        assert!(!(has_sat && has_fx1), "distinctive-vocabulary element is separated: {members:?}");
    }
}

#[test]
fn k_is_validated_and_clamped() {
    let (_o, _e, code) = run_clusters(&["--k", "0"]);
    assert_eq!(code, 1, "--k 0 exits 1");
    let d = clusters_json(&["--k", "100"]);
    assert_eq!(d["k"].as_u64(), Some(6), "k clamped to the element count");
}

#[test]
fn mcp_clusters_tool_matches_cli() {
    use common::Mcp;
    let model = common::fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();

    let tools = mcp.tools_list();
    let tool = tools.get("tools").and_then(|t| t.as_array())
        .and_then(|a| a.iter().find(|t| t.get("name").and_then(|n| n.as_str()) == Some("clusters")))
        .expect("clusters tool advertised");
    assert_eq!(tool.pointer("/annotations/readOnlyHint").and_then(|v| v.as_bool()), Some(true));

    let tool_doc = mcp.call_tool("clusters", serde_json::json!({ "k": 2 }));
    let cli = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m").arg(&model).arg("clusters").arg("--k").arg("2").arg("--json")
        .output().expect("spawn clusters");
    let cli_doc: Value = serde_json::from_slice(&cli.stdout).expect("cli json");
    assert_eq!(tool_doc, cli_doc, "MCP clusters == clusters --k 2 --json");
}
