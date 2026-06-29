//! Integration tests for Bundle B (live + read-only).
//! Realises TC-TRS-MCP-024 (read-only mode), 025 (logging capability),
//! 026 (model-change notifications).

mod common;
use common::*;
use serde_json::json;

fn tool_names(list: &serde_json::Value) -> Vec<String> {
    list.get("tools")
        .and_then(|t| t.as_array())
        .expect("tools array")
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()).map(String::from))
        .collect()
}

// ---- TC-TRS-MCP-024: read-only mode -----------------------------------------

#[test]
fn read_only_hides_write_tools() {
    let model = fixture_copy();
    let mut mcp = Mcp::start_with_args(&model, &["--read-only"]);
    mcp.initialize();
    let names = tool_names(&mcp.tools_list());
    for w in ["create_element", "update_element", "move_element", "delete_element", "apply_changes"] {
        assert!(!names.contains(&w.to_string()), "{w} must be absent in --read-only; got {names:?}");
    }
    for r in ["get_element", "search", "validate"] {
        assert!(names.contains(&r.to_string()), "{r} still available in --read-only");
    }
}

#[test]
fn default_mode_exposes_write_tools() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let names = tool_names(&mcp.tools_list());
    assert!(names.contains(&"create_element".to_string()), "write tools present by default");
}

// ---- TC-TRS-MCP-025: logging capability -------------------------------------

#[test]
fn logging_capability_is_advertised() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    let res = mcp.initialize();
    assert!(
        res.get("capabilities").and_then(|c| c.get("logging")).is_some(),
        "logging capability advertised; got {res}"
    );
}

#[test]
fn logging_set_level_is_accepted() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.logging_set_level("info");
    // A success result is an (possibly empty) object, not an error (the harness panics on error).
    assert!(res.is_object() || res.is_null(), "setLevel returns a success result; got {res}");
}

// ---- TC-TRS-MCP-026: model-change notifications -----------------------------

#[test]
fn resources_capability_declares_listchanged_and_subscribe() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    let res = mcp.initialize();
    let resources = res.get("capabilities").and_then(|c| c.get("resources")).expect("resources capability");
    assert_eq!(resources.get("listChanged").and_then(|b| b.as_bool()), Some(true), "listChanged advertised");
    assert_eq!(resources.get("subscribe").and_then(|b| b.as_bool()), Some(true), "subscribe advertised");
}

#[test]
fn committed_write_emits_list_changed() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    mcp.call_tool(
        "update_element",
        json!({"ref": "REQ-FX-001", "fields": {"status": "approved"}, "dry_run": false}),
    );
    // A follow-up request guarantees any post-response notification has been read.
    let _ = mcp.call_tool("get_element", json!({"ref": "REQ-FX-001"}));
    assert!(
        mcp.saw_notification("notifications/resources/list_changed"),
        "a list_changed notification was delivered after a committed write; saw {:?}",
        mcp.notifications.iter().filter_map(|n| n.get("method").and_then(|m| m.as_str())).collect::<Vec<_>>()
    );
}
