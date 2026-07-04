//! Integration tests for `syscribe topics` and the MCP `topics` tool
//! (TC-TRS-SEARCH-002 → REQ-TRS-SEARCH-002).

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

fn fixture_model() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/model")
}

fn run_topics(extra: &[&str]) -> (String, String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m").arg(fixture_model())
        .arg("topics").args(extra)
        .output().expect("spawn topics");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

fn topics_json(extra: &[&str]) -> Value {
    let mut args = vec!["--json"];
    args.extend_from_slice(extra);
    let (stdout, stderr, code) = run_topics(&args);
    assert_eq!(code, 0, "topics --json exits 0 (stderr: {stderr})");
    serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("topics --json valid: {e}\n{stdout}"))
}

const STOPWORDS: [&str; 5] = ["the", "shall", "and", "is", "a"];

#[test]
fn each_package_gets_distinctive_terms() {
    let d = topics_json(&[]);
    let packages = d["packages"].as_object().expect("packages object");
    assert!(!packages.is_empty(), "at least one package");
    for (_pkg, terms) in packages {
        let arr = terms.as_array().unwrap();
        // ordered by descending score
        let scores: Vec<f64> = arr.iter().map(|t| t["score"].as_f64().unwrap()).collect();
        for w in scores.windows(2) {
            assert!(w[0] >= w[1], "terms ordered by descending score");
        }
        // no stopwords
        for t in arr {
            let term = t["term"].as_str().unwrap();
            assert!(!STOPWORDS.contains(&term), "stopword '{term}' present in topics");
        }
    }
}

#[test]
fn top_bounds_the_term_count() {
    let d = topics_json(&["--top", "3"]);
    for (_pkg, terms) in d["packages"].as_object().unwrap() {
        assert!(terms.as_array().unwrap().len() <= 3, "at most 3 terms per package");
    }
}

#[test]
fn type_selects_element_type_and_spans_packages() {
    // FeatureDefs live in Features/ and Features/Link/ → two packages.
    let d = topics_json(&["--type", "FeatureDef"]);
    let packages = d["packages"].as_object().unwrap();
    assert!(packages.contains_key("Features"), "Features package present: {packages:?}");
    assert!(packages.contains_key("Features::Link"), "Features::Link package present");
}

#[test]
fn unresolvable_config_is_usage_error() {
    let (_o, _e, code) = run_topics(&["--config", "bogus", "--json"]);
    assert_eq!(code, 1, "unresolvable --config exits 1");
}

#[test]
fn mcp_topics_tool_matches_cli() {
    use common::Mcp;
    let model = common::fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();

    let tools = mcp.tools_list();
    let tool = tools.get("tools").and_then(|t| t.as_array())
        .and_then(|a| a.iter().find(|t| t.get("name").and_then(|n| n.as_str()) == Some("topics")))
        .expect("topics tool advertised");
    assert_eq!(tool.pointer("/annotations/readOnlyHint").and_then(|v| v.as_bool()), Some(true));

    let tool_doc = mcp.call_tool("topics", serde_json::json!({}));
    let cli = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m").arg(&model).arg("topics").arg("--json")
        .output().expect("spawn topics");
    let cli_doc: Value = serde_json::from_slice(&cli.stdout).expect("cli json");
    assert_eq!(tool_doc, cli_doc, "MCP topics == topics --json");
}
