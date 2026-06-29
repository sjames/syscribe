//! Integration tests for the verification-evidence MCP tools (Feature A).
//! Realises TC-TRS-MCP-035 (contract), 036 (ingest_results), 037 (coverage_matrix),
//! 038 (coverage_gaps), 039 (evidence).

mod common;
use common::*;
use serde_json::json;

/// A cargo-json (libtest) report marking the fixture TC-FX-001's test function passed.
/// TC-FX-001 declares testFunctions[].function = "fx::tests::req_fx_001" (leaf "req_fx_001").
const PASS_REPORT: &str = r#"{"type":"test","name":"fx::tests::req_fx_001","event":"ok"}"#;

fn sidecar(mcp: &Mcp) -> std::path::PathBuf {
    mcp.model_root.join(".syscribe/results.json")
}

// ---- TC-TRS-MCP-035: common contract ----------------------------------------

#[test]
fn read_tool_returns_structured_json() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("coverage_matrix", json!({}));
    assert!(res.is_object(), "coverage_matrix returns a structured object; got {res}");
    assert!(res.get("coverage").is_some(), "result carries a coverage rollup");
}

#[test]
fn ingest_is_dry_run_by_default() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("ingest_results", json!({"format": "cargo-json", "content": PASS_REPORT}));
    assert_ne!(res.get("written").and_then(|w| w.as_bool()), Some(true), "default is dry-run (not written)");
    assert!(!sidecar(&mcp).exists(), "dry-run writes no sidecar");
}

// ---- TC-TRS-MCP-036: ingest_results -----------------------------------------

#[test]
fn ingest_dry_run_reports_delta_without_writing() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("ingest_results", json!({"format": "cargo-json", "content": PASS_REPORT, "dry_run": true}));
    assert_eq!(res.get("written").and_then(|w| w.as_bool()), Some(false));
    assert!(res.get("delta").is_some() || res.get("verdictDelta").is_some(), "a verdict delta is reported; got {res}");
    assert!(!sidecar(&mcp).exists(), "no sidecar written on dry-run");
}

#[test]
fn ingest_commit_writes_sidecar() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("ingest_results", json!({"format": "cargo-json", "content": PASS_REPORT, "dry_run": false}));
    assert_eq!(res.get("written").and_then(|w| w.as_bool()), Some(true), "commit reports written: {res}");
    assert!(sidecar(&mcp).exists(), ".syscribe/results.json written on commit");
}

#[test]
fn ingest_malformed_report_errors_and_leaves_sidecar() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool_raw("ingest_results", json!({"format": "cargo-json", "content": "not a report at all", "dry_run": false}));
    assert_eq!(res.get("isError").and_then(|e| e.as_bool()), Some(true), "malformed report errors: {res}");
    assert!(!sidecar(&mcp).exists(), "sidecar left unwritten on error");
}

// ---- TC-TRS-MCP-037: coverage_matrix ----------------------------------------

#[test]
fn coverage_matrix_has_grid_and_rollup() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("coverage_matrix", json!({}));
    assert!(res.get("columns").is_some(), "has columns");
    assert!(res.get("rows").and_then(|r| r.as_array()).is_some(), "has rows");
    assert!(res.get("coverage").is_some(), "has coverage rollup");
}

#[test]
fn coverage_matrix_upgrades_to_passing_after_ingest() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    mcp.call_tool("ingest_results", json!({"format": "cargo-json", "content": PASS_REPORT, "dry_run": false}));
    let res = mcp.call_tool("coverage_matrix", json!({}));
    let rows = res.get("rows").and_then(|r| r.as_array()).expect("rows");
    let row = rows.iter().find(|r| r.get("id").and_then(|i| i.as_str()) == Some("REQ-FX-001")).expect("REQ-FX-001 row");
    let cells = row.get("cells").and_then(|c| c.as_object()).expect("cells");
    assert!(
        cells.values().any(|v| v.as_str() == Some("passing")),
        "REQ-FX-001 cell is 'passing' after ingest; got {cells:?}"
    );
}

// ---- TC-TRS-MCP-045: unified classifier (coverage <-> coverage_matrix) ------

fn cell_for(matrix: &serde_json::Value, req: &str) -> Option<String> {
    let row = matrix.get("rows")?.as_array()?.iter().find(|r| r.get("id").and_then(|i| i.as_str()) == Some(req))?;
    let cells = row.get("cells")?.as_object()?;
    cells.values().next().and_then(|v| v.as_str()).map(String::from)
}

fn id_list(v: &serde_json::Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(|a| a.as_array())
        .map(|a| a.iter().filter_map(|e| e.get("id").and_then(|i| i.as_str()).map(String::from)).collect())
        .unwrap_or_default()
}

#[test]
fn draft_only_linked_requirement_is_planned_not_verified() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let cov = mcp.call_tool("coverage", json!({}));
    let mtx = mcp.call_tool("coverage_matrix", json!({}));

    // REQ-FXPLAN-001 is linked only by a draft TestCase -> planned, never verified.
    let planned = id_list(&cov, "planned");
    let unverified = id_list(&cov, "unverifiedLeaves");
    assert!(planned.contains(&"REQ-FXPLAN-001".to_string()), "draft-linked req is in the planned set; got planned={planned:?}");
    assert!(!unverified.contains(&"REQ-FXPLAN-001".to_string()), "planned req is not an unverified leaf");
    assert_eq!(cell_for(&mtx, "REQ-FXPLAN-001").as_deref(), Some("planned"), "matrix cell is planned");

    // REQ-FX-001 is linked by a non-draft TestCase -> verified; matrix cell never a gap.
    assert!(!planned.contains(&"REQ-FX-001".to_string()), "non-draft-linked req is not planned");
    assert!(!unverified.contains(&"REQ-FX-001".to_string()), "non-draft-linked req is verified, not unverified");
    let c = cell_for(&mtx, "REQ-FX-001");
    assert!(matches!(c.as_deref(), Some("covered") | Some("passing")), "REQ-FX-001 cell is covered/passing, not {c:?}");
}

// ---- TC-TRS-MCP-038: coverage_gaps ------------------------------------------

#[test]
fn coverage_gaps_reports_uncovered_approved_requirement() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("coverage_gaps", json!({}));
    let rows = res.get("gaps").or_else(|| res.get("rows")).and_then(|r| r.as_array()).expect("gap rows");
    // REQ-FX-003 is approved with no verifying TestCase -> uncovered.
    let row = rows.iter().find(|r| {
        let refv = r.get("ref").and_then(|x| x.as_str()).unwrap_or("");
        refv.contains("REQ-FX-003")
    }).expect("REQ-FX-003 gap row");
    assert_eq!(row.get("class").and_then(|c| c.as_str()), Some("uncovered"), "classified uncovered; got {row}");
}

// ---- TC-TRS-MCP-039: evidence -----------------------------------------------

#[test]
fn evidence_verdict_unknown_without_results() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("evidence", json!({"ref": "REQ-FX-001"}));
    let s = serde_json::to_string(&res).unwrap();
    assert!(s.contains("TC-FX-001"), "evidence lists the verifying test case");
    assert!(s.contains("req_fx_001"), "evidence lists the test function");
    assert!(s.contains("unknown"), "verdict is unknown with no results sidecar; got {s}");
}

#[test]
fn evidence_verdict_pass_after_ingest() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    mcp.call_tool("ingest_results", json!({"format": "cargo-json", "content": PASS_REPORT, "dry_run": false}));
    let res = mcp.call_tool("evidence", json!({"ref": "REQ-FX-001"}));
    let s = serde_json::to_string(&res).unwrap();
    assert!(s.contains("pass"), "verdict is pass after ingesting a passing report; got {s}");
}
