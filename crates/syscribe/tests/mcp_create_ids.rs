//! GH #185: id-identified files are `<id>.md`; packages are `_index.md`.

mod common;
use common::*;
use serde_json::json;

fn req_fields() -> serde_json::Value {
    json!({"name": "N", "status": "draft", "reqDomain": "software", "reqClass": "system"})
}

fn written(v: &serde_json::Value) -> Option<bool> {
    v.get("written").and_then(|w| w.as_bool())
}

#[test]
fn create_requirement_with_parent_and_id_writes_id_file() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool(
        "create_element",
        json!({
            "parent": "Requirements::Safety", "type": "Requirement",
            "fields": {"id": "REQ-ENG-SAFE-006", "name": "Safe", "status": "draft", "reqDomain": "software", "reqClass": "system"},
            "dry_run": false
        }),
    );
    assert_eq!(written(&res), Some(true), "committed: {res}");
    assert!(model.join("Requirements/Safety/REQ-ENG-SAFE-006.md").exists());
    assert_eq!(res.get("qname").and_then(|q| q.as_str()), Some("Requirements::Safety::REQ-ENG-SAFE-006"));
    let got = mcp.call_tool("get_element", json!({"ref": "REQ-ENG-SAFE-006"}));
    assert_eq!(got.get("qname").and_then(|q| q.as_str()), Some("Requirements::Safety::REQ-ENG-SAFE-006"));
}

#[test]
fn create_with_qname_ending_in_id_and_auto_allocation() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool(
        "create_element",
        json!({"qname": "Requirements::REQ-ENG-SAFE-007", "type": "Requirement", "fields": req_fields(), "dry_run": false}),
    );
    assert_eq!(written(&res), Some(true), "committed: {res}");
    assert!(model.join("Requirements/REQ-ENG-SAFE-007.md").exists());

    let res = mcp.call_tool(
        "create_element",
        json!({"parent": "Requirements", "type": "Requirement", "fields": req_fields(), "dry_run": false}),
    );
    assert_eq!(written(&res), Some(true), "committed: {res}");
    let id = res.get("id").and_then(|i| i.as_str()).unwrap();
    assert!(model.join(format!("Requirements/{id}.md")).exists(), "auto id names the file: {res}");
}

#[test]
fn dry_run_reports_the_planned_path_without_writing() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool(
        "create_element",
        json!({"parent": "Requirements", "type": "Requirement", "fields": req_fields()}),
    );
    assert_eq!(written(&res), Some(false), "dry run: {res}");
    let path = res.get("path").and_then(|p| p.as_str()).expect("path reported");
    assert!(path.starts_with("Requirements/REQ-") && path.ends_with(".md"), "{path}");
    assert!(!model.join(path).exists());
}

#[test]
fn create_with_invalid_id_or_no_target_is_refused() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool(
        "create_element",
        json!({"parent": "Requirements", "type": "Requirement", "fields": {"id": "bad id"}, "dry_run": false}),
    );
    assert_eq!(written(&res), Some(false), "{res}");
    let res = mcp.call_tool("create_element", json!({"type": "Requirement", "dry_run": false}));
    assert_eq!(written(&res), Some(false), "{res}");
}

#[test]
fn create_package_writes_index_md() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool(
        "create_element",
        json!({"qname": "Design", "type": "Package", "fields": {"name": "Design"}, "dry_run": false}),
    );
    assert_eq!(written(&res), Some(true), "committed: {res}");
    assert!(model.join("Design/_index.md").exists());
    assert!(!model.join("Design.md").exists());
}

#[test]
fn move_to_id_destination_renames_the_file() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let r = mcp.call_tool(
        "create_element",
        json!({"qname": "Requirements::REQ_OLD", "type": "Requirement", "fields": req_fields(), "dry_run": false}),
    );
    assert_eq!(written(&r), Some(true), "{r}");
    let res = mcp.call_tool(
        "move_element",
        json!({"ref": "Requirements::REQ_OLD", "dest": "Requirements::REQ-NEW-001", "dry_run": false}),
    );
    assert_eq!(written(&res), Some(true), "committed: {res}");
    assert!(model.join("Requirements/REQ-NEW-001.md").exists());
    assert!(!model.join("Requirements/REQ_OLD.md").exists());
}

#[test]
fn apply_changes_create_supports_parent_and_package() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool(
        "apply_changes",
        json!({
            "dry_run": false,
            "operations": [
                {"op": "create", "qname": "Plans", "type": "Package", "fields": {"name": "Plans"}},
                {"op": "create", "parent": "Plans", "type": "Requirement",
                 "fields": {"id": "REQ-PLAN-001", "name": "P", "status": "draft", "reqDomain": "software", "reqClass": "system"}}
            ]
        }),
    );
    assert_eq!(written(&res), Some(true), "batch commits: {res}");
    assert!(model.join("Plans/_index.md").exists());
    assert!(model.join("Plans/REQ-PLAN-001.md").exists());
}
