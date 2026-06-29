//! Integration tests for `syscribe mcp` resources & prompts.
//! Realises TC-TRS-MCP-004.

mod common;
use common::*;

#[test]
fn spec_sections_listed_and_readable_as_resources() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();

    let list = mcp.resources_list();
    let resources = list.get("resources").and_then(|r| r.as_array()).expect("resources array");
    let spec_uri = resources
        .iter()
        .filter_map(|r| r.get("uri").and_then(|u| u.as_str()))
        .find(|u| u.starts_with("syscribe://spec/"))
        .expect("a syscribe://spec/ resource is listed")
        .to_string();

    let read = mcp.resources_read(&spec_uri);
    let text = read
        .get("contents")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .and_then(|item| item.get("text"))
        .and_then(|t| t.as_str())
        .expect("resource read returns text");
    assert!(!text.is_empty(), "spec section has content");
}

#[test]
fn authoring_prompt_is_exposed() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();

    let list = mcp.prompts_list();
    let prompts = list.get("prompts").and_then(|p| p.as_array()).expect("prompts array");
    let name = prompts
        .iter()
        .filter_map(|p| p.get("name").and_then(|n| n.as_str()))
        .find(|n| n.contains("create-model") || n.contains("create_model"))
        .expect("a create-model prompt is listed")
        .to_string();

    let got = mcp.prompts_get(&name);
    assert!(got.get("messages").is_some(), "prompts/get returns messages");
}
