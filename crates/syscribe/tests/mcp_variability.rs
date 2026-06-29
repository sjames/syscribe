//! Integration tests for the feature-model / projection bundle.
//! Realises TC-TRS-MCP-029 (features/feature_check), 030 (configure),
//! 031 (project), 032 (diff_configs), 033 (why_active).

mod common;
use common::*;
use serde_json::json;

// ---- TC-TRS-MCP-029: features / feature_check -------------------------------

#[test]
fn features_lists_the_model() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("features", json!({}));
    assert_eq!(res.get("hasFeatureModel").and_then(|b| b.as_bool()), Some(true), "feature model present: {res}");
    let qnames: Vec<String> = res
        .get("features")
        .and_then(|f| f.as_array())
        .map(|a| a.iter().filter_map(|x| x.get("qname").and_then(|q| q.as_str()).map(String::from)).collect())
        .unwrap_or_default();
    assert!(qnames.iter().any(|q| q.contains("Features::Link::LoRa")), "LoRa feature listed; got {qnames:?}");
}

#[test]
fn feature_check_deep_reports_not_void() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("feature_check", json!({"deep": true}));
    assert!(res.get("findings").and_then(|f| f.as_array()).is_some(), "findings array; got {res}");
    let void = res.get("deep").and_then(|d| d.get("void")).and_then(|v| v.as_bool());
    assert_eq!(void, Some(false), "the fixture feature model is not void; got {res}");
}

// ---- TC-TRS-MCP-030: configure ----------------------------------------------

#[test]
fn configure_reports_satisfiable() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("configure", json!({"config": "CONF-FX-001"}));
    assert_eq!(res.get("satisfiable").and_then(|b| b.as_bool()), Some(true), "CONF-FX-001 satisfiable: {res}");
    assert!(res.get("free").is_some(), "free list present");
}

// ---- TC-TRS-MCP-031: project ------------------------------------------------

#[test]
fn project_returns_active_set_and_findings() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("project", json!({"config": "CONF-FX-001"}));
    assert_eq!(
        res.get("selection").and_then(|s| s.get("Features::Link::LoRa")).and_then(|b| b.as_bool()),
        Some(true),
        "LoRa selected in the projection; got {res}"
    );
    let active: Vec<String> = res
        .get("active")
        .and_then(|a| a.as_array())
        .map(|a| a.iter().filter_map(|x| x.get("qname").and_then(|q| q.as_str()).map(String::from)).collect())
        .unwrap_or_default();
    assert!(!active.is_empty(), "active set non-empty");
    assert!(res.get("findings").and_then(|f| f.as_array()).is_some(), "projected findings array present");
    assert!(
        !active.iter().any(|q| q.contains("REQ-FXSAT-001")),
        "the satellite-only requirement is not active under the LoRa config; got {active:?}"
    );
}

// ---- TC-TRS-MCP-032: diff_configs -------------------------------------------

#[test]
fn diff_configs_reports_only_in_b() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("diff_configs", json!({"a": "CONF-FX-001", "b": "Features::Link::Sat"}));
    let only_b: Vec<String> = res
        .get("onlyInB")
        .and_then(|a| a.as_array())
        .map(|a| a.iter().filter_map(|x| x.get("id").and_then(|i| i.as_str()).map(String::from)).collect())
        .unwrap_or_default();
    assert!(only_b.iter().any(|i| i == "REQ-FXSAT-001"), "satellite requirement only in B; got {only_b:?}");
}

// ---- TC-TRS-MCP-033: why_active ---------------------------------------------

#[test]
fn why_active_explains_inactive_gated_element() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("why_active", json!({"ref": "REQ-FXSAT-001", "config": "CONF-FX-001"}));
    assert_eq!(res.get("active").and_then(|b| b.as_bool()), Some(false), "gated element inactive: {res}");
    let s = serde_json::to_string(&res).unwrap();
    assert!(s.contains("Features::Link::Sat"), "effective appliesWhen references the gating feature");
}

#[test]
fn why_active_ungated_element_is_active() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("why_active", json!({"ref": "REQ-FX-001", "config": "CONF-FX-001"}));
    assert_eq!(res.get("active").and_then(|b| b.as_bool()), Some(true), "ungated element always active: {res}");
}
