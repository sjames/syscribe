//! Integration tests for the Phase-3 authoring / MCP-native additions.
//! Realises TC-TRS-MCP-012 (describe_type), 013 (template), 014 (explain_finding),
//! 015 (check_ref/next_id), 016 (tool annotations), 017 (element resources +
//! completion), 018 (task prompts), 019 (coverage).

mod common;
use common::*;
use serde_json::json;

fn tool<'a>(list: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    list.get("tools")
        .and_then(|t| t.as_array())
        .expect("tools array")
        .iter()
        .find(|t| t.get("name").and_then(|n| n.as_str()) == Some(name))
        .unwrap_or_else(|| panic!("tool {name} not listed"))
}

// ---- TC-TRS-MCP-012: describe_type ------------------------------------------

#[test]
fn describe_type_reports_fields_and_enums() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("describe_type", json!({"type": "Requirement"}));
    let fields = res.get("fields").and_then(|f| f.as_array()).expect("fields array");
    assert!(!fields.is_empty(), "schema lists fields");
    let status = fields
        .iter()
        .find(|f| f.get("name").and_then(|n| n.as_str()) == Some("status"))
        .expect("status field present");
    let domain: Vec<String> = status
        .get("enum")
        .and_then(|e| e.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    assert!(domain.iter().any(|v| v == "draft"), "status enum domain includes draft; got {domain:?}");
}

// ---- TC-TRS-MCP-013: template -----------------------------------------------

#[test]
fn template_returns_skeleton_for_type() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("template", json!({"type": "Requirement"}));
    let content = res.get("content").and_then(|c| c.as_str()).expect("content string");
    assert!(content.contains("type: Requirement"), "skeleton sets the type; got {content}");
}

#[test]
fn template_unknown_type_is_error() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool_raw("template", json!({"type": "NotAType"}));
    assert_eq!(res.get("isError").and_then(|e| e.as_bool()), Some(true), "unknown type errors: {res}");
}

// ---- TC-TRS-MCP-014: explain_finding ----------------------------------------

#[test]
fn explain_finding_explains_a_known_code() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("explain_finding", json!({"code": "W001"}));
    let explanation = res.get("explanation").and_then(|e| e.as_str()).unwrap_or("");
    assert!(!explanation.is_empty(), "known code has an explanation; got {res}");
}

#[test]
fn explain_finding_unknown_code_is_error() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool_raw("explain_finding", json!({"code": "Z999"}));
    assert_eq!(res.get("isError").and_then(|e| e.as_bool()), Some(true), "unknown code errors");
}

// ---- TC-TRS-MCP-015: check_ref / next_id ------------------------------------

#[test]
fn check_ref_reports_resolution() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let ok = mcp.call_tool("check_ref", json!({"ref": "REQ-FX-001"}));
    assert_eq!(ok.get("resolved").and_then(|r| r.as_bool()), Some(true), "existing ref resolves");
    assert!(ok.get("type").is_some(), "resolved ref reports its type");

    let no = mcp.call_tool("check_ref", json!({"ref": "Nope::DoesNotExist"}));
    assert_eq!(no.get("resolved").and_then(|r| r.as_bool()), Some(false), "missing ref reports resolved=false (not an error)");
}

#[test]
fn next_id_returns_a_free_id_for_prefix() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("next_id", json!({"prefix": "REQ-FX"}));
    let id = res.get("id").and_then(|i| i.as_str()).expect("id returned");
    assert!(id.starts_with("REQ-FX"), "id carries the prefix; got {id}");
    assert_ne!(id, "REQ-FX-001", "id is not an already-taken one");
    assert_ne!(id, "REQ-FX-003", "id is not an already-taken one");
}

// ---- TC-TRS-MCP-016: tool annotations ---------------------------------------

#[test]
fn read_tools_are_annotated_read_only_writes_are_not() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let list = mcp.tools_list();

    for ro in ["get_element", "search", "validate", "trace"] {
        let ann = tool(&list, ro).get("annotations").unwrap_or_else(|| panic!("{ro} has annotations"));
        assert_eq!(
            ann.get("readOnlyHint").and_then(|h| h.as_bool()),
            Some(true),
            "{ro} is annotated readOnlyHint=true"
        );
    }
    for rw in ["create_element", "update_element", "move_element"] {
        let ann = tool(&list, rw).get("annotations");
        let ro = ann.and_then(|a| a.get("readOnlyHint")).and_then(|h| h.as_bool());
        assert_ne!(ro, Some(true), "{rw} must not be readOnlyHint=true");
    }
}

// ---- TC-TRS-MCP-017: element resources + completion -------------------------

#[test]
fn element_is_readable_as_a_resource() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let read = mcp.resources_read("syscribe://element/Requirements::REQ-FX-001");
    let text = read
        .get("contents")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .and_then(|i| i.get("text"))
        .and_then(|t| t.as_str())
        .expect("element resource returns text");
    let detail: serde_json::Value = serde_json::from_str(text).expect("element resource is JSON");
    assert_eq!(detail.get("id").and_then(|i| i.as_str()), Some("REQ-FX-001"));
}

#[test]
fn element_resource_template_is_advertised() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.resource_templates_list();
    let templates = res.get("resourceTemplates").and_then(|t| t.as_array()).expect("resourceTemplates array");
    assert!(
        templates.iter().any(|t| t.get("uriTemplate").and_then(|u| u.as_str()).is_some_and(|u| u.starts_with("syscribe://element/"))),
        "an element resource template is advertised; got {templates:?}"
    );
}

#[test]
fn element_references_are_completable() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.complete_resource("syscribe://element/{qname}", "qname", "REQ-FX");
    let values: Vec<String> = res
        .get("completion")
        .and_then(|c| c.get("values"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    assert!(values.iter().any(|v| v.contains("REQ-FX-001")), "completion suggests matching elements; got {values:?}");
}

// ---- TC-TRS-MCP-018: task prompts -------------------------------------------

#[test]
fn task_prompts_are_listed() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let list = mcp.prompts_list();
    let names: Vec<String> = list
        .get("prompts")
        .and_then(|p| p.as_array())
        .expect("prompts array")
        .iter()
        .filter_map(|p| p.get("name").and_then(|n| n.as_str()).map(String::from))
        .collect();
    for expected in ["add-requirement", "break-down-requirement", "add-testcase-for", "traceability-review"] {
        assert!(names.contains(&expected.to_string()), "prompt {expected} listed; got {names:?}");
    }
}

#[test]
fn add_requirement_prompt_carries_conventions() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let got = mcp.prompts_get("add-requirement");
    let text = serde_json::to_string(&got).unwrap();
    assert!(text.contains("derivedFrom") && text.contains("breakdownAdr"), "prompt references the conventions");
}

// ---- TC-TRS-MCP-019: coverage -----------------------------------------------

#[test]
fn coverage_partitions_leaf_gaps_and_parents_missing_integration_tests() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("coverage", json!({}));

    let verified = res.get("verifiedCount").and_then(|c| c.as_u64()).expect("verifiedCount");
    assert!(verified >= 1, "at least one verified requirement (REQ-FX-001)");

    let ids = |key: &str| -> Vec<String> {
        res.get(key)
            .and_then(|u| u.as_array())
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|u| u.get("id").and_then(|i| i.as_str()).map(String::from))
            .collect()
    };

    // Leaf gaps: leaf requirements with no verifying TestCase.
    let leaves = ids("unverifiedLeaves");
    assert!(leaves.contains(&"REQ-FX-003".to_string()), "REQ-FX-003 is an unverified leaf; got {leaves:?}");
    assert!(leaves.contains(&"REQ-FXCHILD-001".to_string()), "the leaf child is an unverified leaf; got {leaves:?}");
    assert!(!leaves.contains(&"REQ-FXPARENT-001".to_string()), "a parent must not be listed as a leaf gap");

    // Parents missing an integration test: REQ-FXPARENT-001 has only a unit (L2) TC.
    let parents = ids("parentsMissingIntegrationTest");
    assert!(
        parents.contains(&"REQ-FXPARENT-001".to_string()),
        "parent with only a unit-level test still needs an integration test; got {parents:?}"
    );
    assert!(!parents.contains(&"REQ-FX-003".to_string()), "a leaf must not be listed as a parent gap");

    // Entries carry qname+id; parents carry a child count.
    for key in ["unverifiedLeaves", "parentsMissingIntegrationTest"] {
        for e in res.get(key).and_then(|u| u.as_array()).unwrap_or(&vec![]) {
            assert!(e.get("qname").is_some() && e.get("id").is_some(), "{key} entries carry qname+id");
        }
    }
    assert!(
        res.get("parentsMissingIntegrationTest")
            .and_then(|a| a.as_array()).and_then(|a| a.first())
            .and_then(|e| e.get("childCount")).is_some(),
        "parent entries carry a childCount"
    );
}
