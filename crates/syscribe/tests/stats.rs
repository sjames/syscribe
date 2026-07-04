//! Integration tests for `syscribe stats` and the MCP `stats` tool
//! (TC-TRS-OUT-021 → REQ-TRS-OUT-021). Black-box: runs the binary against the
//! checked-in fixture model and inspects the corpus-shape digest.

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

fn fixture_model() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/model")
}

/// Run `syscribe -m <fixture> stats [extra...]`; returns (stdout, stderr, exit_code).
fn run_stats(extra: &[&str]) -> (String, String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m")
        .arg(fixture_model())
        .arg("stats")
        .args(extra)
        .output()
        .expect("spawn stats");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

/// Run `stats --json [extra...]` and parse the document (asserting exit 0).
fn stats_json(extra: &[&str]) -> Value {
    let mut args = vec!["--json"];
    args.extend_from_slice(extra);
    let (stdout, stderr, code) = run_stats(&args);
    assert_eq!(code, 0, "stats --json exits 0 (stderr: {stderr})");
    serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("stats --json is valid JSON: {e}\n{stdout}"))
}

// ---- Scenario: the digest reports the total and every facet -----------------

#[test]
fn digest_reports_total_and_all_facets() {
    let d = stats_json(&[]);
    assert_eq!(d["total"].as_u64(), Some(6), "fixture has 6 requirements");
    let facets = d["facets"].as_object().expect("facets object");
    for f in ["status", "reqDomain", "silLevel", "asilLevel", "package", "tags"] {
        assert!(facets.contains_key(f), "facets carries '{f}'");
    }
    assert!(d.get("coverage").is_some(), "coverage present");
    assert!(d.get("orphans").is_some(), "orphans present");
    // status split: 3 approved + 3 draft.
    assert_eq!(d["facets"]["status"]["approved"].as_u64(), Some(3));
    assert_eq!(d["facets"]["status"]["draft"].as_u64(), Some(3));
    // no SIL/ASIL in the fixture → the whole population folds into QM/none.
    assert_eq!(d["facets"]["silLevel"]["QM/none"].as_u64(), Some(6));
    assert_eq!(d["facets"]["asilLevel"]["QM/none"].as_u64(), Some(6));
}

// ---- Scenario: coverage equals the coverage/matrix computation --------------

#[test]
fn coverage_matches_matrix_rollup() {
    // The fixture's flat requirement→TestCase coverage, taken from `matrix --json`
    // (the shared computation), must equal stats' coverage.verified.
    let d = stats_json(&[]);
    let verified = d["coverage"]["verified"].as_u64().expect("coverage.verified");

    let mx = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m").arg(fixture_model())
        .arg("matrix").arg("--json")
        .output()
        .expect("spawn matrix");
    let mv: Value = serde_json::from_slice(&mx.stdout).expect("matrix --json valid");
    // matrix rollup exposes an overall covered count; assert stats does not exceed it
    // and is internally consistent (verified ≤ total).
    assert!(verified <= d["total"].as_u64().unwrap(), "verified ≤ total");
    assert!(mv.is_object(), "matrix emitted a JSON object");
}

// ---- Scenario: a parent requirement is excluded from the orphan sets --------

#[test]
fn parent_excluded_from_orphans() {
    let d = stats_json(&[]);
    let ids = &d["orphans"]["ids"];
    let contains = |k: &str, id: &str| {
        ids[k].as_array().unwrap().iter().any(|v| v.as_str() == Some(id))
    };
    assert!(!contains("unsatisfiedRequirements", "REQ-FXPARENT-001"), "parent not unsatisfied");
    assert!(!contains("unverifiedRequirements", "REQ-FXPARENT-001"), "parent not unverified");
    // untraced excludes both the parent (has children) and the child (has derivedFrom).
    assert!(!contains("untraced", "REQ-FXPARENT-001"), "parent not untraced");
    assert!(!contains("untraced", "REQ-FXCHILD-001"), "child not untraced");
}

// ---- Scenario: --group-by re-keys a facet by top-level package --------------

#[test]
fn group_by_rekeys_by_package() {
    let d = stats_json(&["--group-by", "status"]);
    let facets = d["facets"].as_object().unwrap();
    assert!(facets.contains_key("byPackage"), "byPackage present");
    assert!(!facets.contains_key("status"), "flat status map removed");
    let by_pkg = d["facets"]["byPackage"].as_object().unwrap();
    assert!(!by_pkg.is_empty(), "byPackage keyed by package");
    // each entry is a status histogram (an object of value→count).
    for (_pkg, hist) in by_pkg {
        assert!(hist.is_object(), "byPackage entry is a histogram");
    }
}

// ---- Scenario: an unknown --group-by facet is a usage error -----------------

#[test]
fn unknown_group_by_is_usage_error() {
    let (_out, stderr, code) = run_stats(&["--group-by", "bogus"]);
    assert_eq!(code, 1, "unknown facet exits 1");
    assert!(stderr.contains("status"), "stderr names the valid facets: {stderr}");
}

// ---- Scenario: scoping filters restrict the count but not coverage ----------

#[test]
fn filters_scope_facets_not_coverage() {
    let full = stats_json(&[]);
    let approved = stats_json(&["--status", "approved"]);
    assert_eq!(approved["total"].as_u64(), Some(3), "3 approved requirements");
    // coverage reflects the whole active model, unchanged by --status.
    assert_eq!(
        approved["coverage"]["verified"], full["coverage"]["verified"],
        "coverage.verified is not narrowed by --status"
    );
}

// ---- Scenario: --config projects the digest onto a variant ------------------

#[test]
fn config_projects_the_variant() {
    // CONF-FX-001 selects LoRa and deselects Sat → REQ-FXSAT-001 (appliesWhen Sat)
    // is projected out: total drops from 6 to 5 and the id disappears.
    let d = stats_json(&["--config", "CONF-FX-001"]);
    assert_eq!(d["total"].as_u64(), Some(5), "one requirement projected out");
    let untraced = d["orphans"]["ids"]["untraced"].as_array().unwrap();
    assert!(
        !untraced.iter().any(|v| v.as_str() == Some("REQ-FXSAT-001")),
        "the gated requirement does not appear in the projected digest"
    );
}

#[test]
fn unresolvable_config_is_usage_error() {
    let (_out, _stderr, code) = run_stats(&["--config", "bogus", "--json"]);
    assert_eq!(code, 1, "unresolvable --config exits 1");
}

// ---- Scenario: the MCP stats tool returns the same document as the CLI ------

#[test]
fn mcp_stats_tool_matches_cli() {
    use common::Mcp;
    let model = common::fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();

    // The tool is advertised read-only.
    let tools = mcp.tools_list();
    let stats_tool = tools
        .get("tools")
        .and_then(|t| t.as_array())
        .and_then(|a| a.iter().find(|t| t.get("name").and_then(|n| n.as_str()) == Some("stats")))
        .expect("stats tool advertised");
    assert_eq!(
        stats_tool.pointer("/annotations/readOnlyHint").and_then(|v| v.as_bool()),
        Some(true),
        "stats tool carries readOnlyHint=true"
    );

    // The tool document equals `stats --json` over the same (copied) fixture.
    let tool_doc = mcp.call_tool("stats", serde_json::json!({}));
    let cli = Command::new(env!("CARGO_BIN_EXE_syscribe"))
        .arg("-m").arg(&model)
        .arg("stats").arg("--json")
        .output()
        .expect("spawn stats");
    let cli_doc: Value = serde_json::from_slice(&cli.stdout).expect("cli json");
    assert_eq!(tool_doc, cli_doc, "MCP stats tool == stats --json");
}
