//! Integration tests for the `syscribe mcp` read tools.
//! Realises TC-TRS-MCP-001 (handshake), TC-TRS-MCP-002 (store + reload),
//! TC-TRS-MCP-003 (token-efficient read tools).

mod common;
use common::*;
use serde_json::json;

// ---- TC-TRS-MCP-001: initialize handshake & tools listing -------------------

#[test]
fn initialize_handshake_advertises_capabilities() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    let res = mcp.initialize();
    let caps = res.get("capabilities").expect("capabilities present");
    assert!(caps.get("tools").is_some(), "tools capability advertised");
    assert!(caps.get("resources").is_some(), "resources capability advertised");
    assert!(caps.get("prompts").is_some(), "prompts capability advertised");
}

// ---- TC-TRS-MCP-044: tool input-schema validity (regression for v0.28.1) ----

#[test]
fn tool_input_schemas_have_object_property_schemas() {
    // A property whose schema is a bare boolean (`true`, as serde_json::Value emits
    // via schemars) is rejected by strict zod-based MCP clients, failing the whole
    // tools/list. Every property must be an object schema.
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.tools_list();
    for tool in res.get("tools").and_then(|t| t.as_array()).expect("tools array") {
        let name = tool.get("name").and_then(|n| n.as_str()).unwrap_or("?");
        if let Some(props) = tool
            .get("inputSchema")
            .and_then(|s| s.get("properties"))
            .and_then(|p| p.as_object())
        {
            for (prop, schema) in props {
                assert!(
                    schema.is_object(),
                    "tool {name} property '{prop}' has a non-object schema {schema} (breaks zod clients)"
                );
            }
        }
    }
}

#[test]
fn tools_list_includes_core_tools() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.tools_list();
    let names: Vec<String> = res
        .get("tools")
        .and_then(|t| t.as_array())
        .expect("tools array")
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()).map(String::from))
        .collect();
    for expected in ["get_element", "search", "validate", "create_element"] {
        assert!(names.contains(&expected.to_string()), "tool {expected} listed; got {names:?}");
    }
}

// ---- TC-TRS-MCP-002: shared store & reload ----------------------------------

#[test]
fn repeated_reads_are_consistent() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let a = mcp.call_tool("get_element", json!({"ref": "REQ-FX-001", "detail": true}));
    let b = mcp.call_tool("get_element", json!({"ref": "REQ-FX-001", "detail": true}));
    assert_eq!(a, b, "identical reads with no edit");
}

#[test]
fn reload_picks_up_external_addition() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let before = mcp.call_tool("reload", json!({}));
    let count_before = before.get("count").and_then(|c| c.as_u64()).expect("count");

    // External edit: add a new element file directly on disk.
    std::fs::write(
        model.join("Requirements/REQ-FX-002.md"),
        "---\ntype: Requirement\nid: REQ-FX-002\nname: \"Added externally\"\nstatus: draft\nreqDomain: software\nreqClass: system\n---\n\nThe system shall be reloadable.\n",
    )
    .unwrap();

    let after = mcp.call_tool("reload", json!({}));
    let count_after = after.get("count").and_then(|c| c.as_u64()).expect("count");
    assert_eq!(count_after, count_before + 1, "reload sees the added element");

    let got = mcp.call_tool("get_element", json!({"ref": "REQ-FX-002"}));
    assert_eq!(got.get("id").and_then(|i| i.as_str()), Some("REQ-FX-002"));
}

// ---- TC-TRS-MCP-003: token-efficient read tools -----------------------------

#[test]
fn get_element_summary_omits_body_detail_includes_it() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();

    let summary = mcp.call_tool("get_element", json!({"ref": "REQ-FX-001"}));
    assert_eq!(summary.get("id").and_then(|i| i.as_str()), Some("REQ-FX-001"));
    assert!(summary.get("qname").is_some(), "summary carries qname");
    let summary_doc = summary.get("doc").and_then(|d| d.as_str()).unwrap_or("");
    assert!(summary_doc.is_empty(), "summary omits full doc body");

    let detail = mcp.call_tool("get_element", json!({"ref": "REQ-FX-001", "detail": true}));
    let detail_doc = detail.get("doc").and_then(|d| d.as_str()).unwrap_or("");
    assert!(!detail_doc.is_empty(), "detail includes the doc body");
}

#[test]
fn search_is_bounded_and_reports_total() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("search", json!({"query": "fixture", "limit": 1}));
    let results = res.get("results").and_then(|r| r.as_array()).expect("results array");
    assert!(results.len() <= 1, "limit respected");
    assert!(res.get("total").is_some(), "total reported");
    for r in results {
        assert!(r.get("qname").is_some() && r.get("id").is_some(), "results carry qname+id");
    }
}

#[test]
fn reference_resolves_by_id_and_qname() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let by_id = mcp.call_tool("get_element", json!({"ref": "REQ-FX-001"}));
    let by_qname = mcp.call_tool("get_element", json!({"ref": "Requirements::REQ-FX-001"}));
    assert_eq!(by_id.get("qname"), by_qname.get("qname"), "id and qname resolve to same element");
}

#[test]
fn validate_reports_findings_and_validate_element_is_scoped() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();

    // Whole-model validate returns a structured findings list.
    let all = mcp.call_tool("validate", json!({}));
    assert!(
        all.get("findings").and_then(|f| f.as_array()).is_some(),
        "validate returns a findings array; got {all}"
    );

    // Element-scoped validate is bound to a single element reference.
    let scoped = mcp.call_tool("validate_element", json!({"ref": "REQ-FX-001"}));
    assert!(
        scoped.get("findings").and_then(|f| f.as_array()).is_some(),
        "validate_element returns a findings array; got {scoped}"
    );
}

#[test]
fn trace_reports_verifying_test_case() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("trace", json!({"ref": "REQ-FX-001"}));
    let verified_by = res
        .get("verifiedBy")
        .and_then(|v| v.as_array())
        .expect("verifiedBy array");
    assert!(
        verified_by
            .iter()
            .any(|v| v.get("id").and_then(|i| i.as_str()) == Some("TC-FX-001")),
        "trace surfaces the verifying test case; got {res}"
    );
}

#[test]
fn impact_reaches_dependent_via_supertype() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("impact", json!({"ref": "Parts::Base"}));
    let affected = res
        .get("affected")
        .and_then(|a| a.as_array())
        .expect("affected array");
    assert!(!affected.is_empty(), "impact reports affected elements; got {res}");
    assert!(
        affected
            .iter()
            .any(|e| e.get("qname").and_then(|q| q.as_str()).is_some_and(|q| q.contains("Parts::Derived"))),
        "downstream impact reaches the subtype that depends on it; got {res}"
    );
    for e in affected {
        assert!(e.get("qname").is_some(), "each affected entry carries qname");
        assert!(
            e.get("distance").and_then(|d| d.as_u64()).is_some(),
            "each affected entry carries a numeric distance; got {e}"
        );
    }
}

// ---- TC-TRS-MCP-010: curated tool surface -----------------------------------

#[test]
fn excluded_tool_families_are_not_listed() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.tools_list();
    let names: Vec<String> = res
        .get("tools")
        .and_then(|t| t.as_array())
        .expect("tools array")
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()).map(String::from))
        .collect();
    // The report/render family stays off the dedicated tool list (reachable via run_report);
    // feature-model tools (configure, feature_check, diff_configs, …) ARE exposed (Bundle E).
    for excluded in [
        "export", "plantuml", "render", "n2", "matrix", "fmea", "safety-case", "sbom",
        "reqif", "audit", "metrics", "zones",
    ] {
        assert!(
            !names.contains(&excluded.to_string()),
            "excluded command {excluded} must not be a tool; got {names:?}"
        );
    }
}

// ---- TC-TRS-MCP-011: structured tool-error handling -------------------------

#[test]
fn unresolved_reference_errors_and_server_keeps_serving() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();

    // An unresolvable reference returns a tool error result (isError), not a crash.
    let err = mcp.call_tool_raw("get_element", json!({"ref": "Nope::DoesNotExist"}));
    assert_eq!(
        err.get("isError").and_then(|e| e.as_bool()),
        Some(true),
        "unresolved ref flagged as a tool error; got {err}"
    );
    assert!(mcp.is_alive(), "server still running after a tool error");

    // A subsequent valid request is served normally.
    let ok = mcp.call_tool("get_element", json!({"ref": "REQ-FX-001"}));
    assert_eq!(ok.get("id").and_then(|i| i.as_str()), Some("REQ-FX-001"));
}

#[test]
fn graph_query_follows_verifies_edge() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool(
        "graph_query",
        json!({"from": "REQ-FX-001", "edges": ["verifies"], "direction": "in"}),
    );
    let nodes: Vec<String> = res
        .get("nodes")
        .and_then(|n| n.as_array())
        .map(|a| a.iter().filter_map(|x| {
            x.as_str().map(String::from).or_else(|| x.get("qname").and_then(|q| q.as_str()).map(String::from))
        }).collect())
        .unwrap_or_default();
    assert!(nodes.iter().any(|n| n.contains("TC-FX-001")), "verifying test case reached; got {nodes:?}");
}

// ---- TC-TRS-MCP-020: remaining read-tool coverage ---------------------------

#[test]
fn list_by_type_enumerates_with_total() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("list_by_type", json!({"type": "Requirement"}));
    let items: Vec<String> = res
        .get("items")
        .and_then(|i| i.as_array())
        .expect("items array")
        .iter()
        .filter_map(|i| i.get("id").and_then(|v| v.as_str()).map(String::from))
        .collect();
    assert!(items.iter().any(|i| i == "REQ-FX-001"), "REQ-FX-001 listed; got {items:?}");
    assert!(res.get("total").is_some(), "total reported");
}

#[test]
fn tree_returns_containment_subtree() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("tree", json!({"depth": 3}));
    // A nested structure with at least one child carrying a qname.
    let s = serde_json::to_string(&res).unwrap();
    assert!(s.contains("\"qname\""), "tree carries qnames; got {s}");
    assert!(s.contains("children"), "tree is nested");
}

#[test]
fn neighbors_returns_one_hop_inbound() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("neighbors", json!({"ref": "REQ-FX-001", "direction": "in"}));
    let inbound = res.get("in").and_then(|i| i.as_array()).expect("in array");
    assert!(
        inbound.iter().any(|n| n.get("qname").and_then(|q| q.as_str()).is_some_and(|q| q.contains("TC-FX-001"))),
        "inbound neighbours include the verifying test case; got {inbound:?}"
    );
}

#[test]
fn get_element_fields_projection_limits_keys() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool(
        "get_element",
        json!({"ref": "REQ-FX-001", "detail": true, "fields": ["status"]}),
    );
    // The projected field is present; a non-projected detail field (doc body) is not.
    let fm = res.get("frontmatter").unwrap_or(&res);
    assert!(fm.get("status").is_some(), "projected field present");
    let doc = res.get("doc").and_then(|d| d.as_str()).unwrap_or("");
    assert!(doc.is_empty(), "non-projected fields excluded under a fields projection");
}
