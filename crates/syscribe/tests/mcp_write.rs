//! Integration tests for the `syscribe mcp` guarded-write tools.
//! Realises TC-TRS-MCP-005 (create), TC-TRS-MCP-006 (update),
//! TC-TRS-MCP-007 (move), TC-TRS-MCP-008 (dry-run / delta / commit gate).

mod common;
use common::*;
use serde_json::json;

// ---- TC-TRS-MCP-005: create_element -----------------------------------------

#[test]
fn create_element_commits_a_new_file() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();

    let res = mcp.call_tool(
        "create_element",
        json!({"qname": "Parts::Sensor", "type": "PartDef", "dry_run": false}),
    );
    assert_eq!(res.get("written").and_then(|w| w.as_bool()), Some(true), "committed: {res}");
    assert!(model.join("Parts/Sensor.md").exists(), "new file created on disk");

    let got = mcp.call_tool("get_element", json!({"ref": "Parts::Sensor"}));
    assert_eq!(got.get("qname").and_then(|q| q.as_str()), Some("Parts::Sensor"));
}

#[test]
fn create_requirement_auto_allocates_stable_id() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();

    let res = mcp.call_tool(
        "create_element",
        json!({
            "qname": "Requirements::REQ_NEW",
            "type": "Requirement",
            "fields": {"name": "Auto id requirement", "status": "draft", "reqDomain": "software", "reqClass": "system"},
            "dry_run": false
        }),
    );
    assert_eq!(res.get("written").and_then(|w| w.as_bool()), Some(true), "committed: {res}");
    let id = res.get("id").and_then(|i| i.as_str()).expect("id reported");
    assert!(id.starts_with("REQ-"), "auto-allocated requirement id, got {id}");
}

#[test]
fn create_existing_qname_is_refused() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let before = std::fs::read(model.join("Parts/Base.md")).unwrap();

    let res = mcp.call_tool(
        "create_element",
        json!({"qname": "Parts::Base", "type": "PartDef", "dry_run": false}),
    );
    assert_eq!(res.get("written").and_then(|w| w.as_bool()), Some(false), "refused: {res}");
    let after = std::fs::read(model.join("Parts/Base.md")).unwrap();
    assert_eq!(before, after, "existing file left unchanged");
}

// ---- TC-TRS-MCP-006: update_element -----------------------------------------

#[test]
fn update_preserves_unknown_key_and_body() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();

    let original = std::fs::read_to_string(model.join("Parts/Base.md")).unwrap();
    let original_body = original.split("---").nth(2).unwrap().to_string();

    let res = mcp.call_tool(
        "update_element",
        json!({"ref": "Parts::Base", "fields": {"isAbstract": true}, "dry_run": false}),
    );
    assert_eq!(res.get("written").and_then(|w| w.as_bool()), Some(true), "committed: {res}");

    let updated = std::fs::read_to_string(model.join("Parts/Base.md")).unwrap();
    assert!(updated.contains("customKey: keepme"), "unknown frontmatter key preserved");
    assert!(updated.contains("isAbstract: true"), "requested field applied");
    let updated_body = updated.split("---").nth(2).unwrap().to_string();
    assert_eq!(updated_body, original_body, "Markdown body unchanged");
}

// ---- TC-TRS-MCP-007: move_element -------------------------------------------

#[test]
fn move_rewrites_inbound_reference() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();

    let res = mcp.call_tool(
        "move_element",
        json!({"ref": "Parts::Base", "dest": "Parts::CoreBase", "dry_run": false}),
    );
    assert_eq!(res.get("written").and_then(|w| w.as_bool()), Some(true), "committed: {res}");
    assert!(model.join("Parts/CoreBase.md").exists(), "moved to new path");
    assert!(!model.join("Parts/Base.md").exists(), "old path gone");

    let derived = std::fs::read_to_string(model.join("Parts/Derived.md")).unwrap();
    assert!(derived.contains("Parts::CoreBase"), "inbound supertype reference rewritten");
}

#[test]
fn move_to_invalid_qname_is_refused() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool(
        "move_element",
        json!({"ref": "Parts::Base", "dest": "Parts::not a name", "dry_run": false}),
    );
    assert_eq!(res.get("written").and_then(|w| w.as_bool()), Some(false), "refused");
    assert!(model.join("Parts/Base.md").exists(), "source unchanged");
}

// ---- TC-TRS-MCP-008: dry-run / delta / commit gate --------------------------

#[test]
fn dry_run_leaves_disk_unchanged() {
    let model = fixture_copy();
    let before = dir_hash(&model);
    let mut mcp = Mcp::start(&model);
    mcp.initialize();

    // No dry_run argument => defaults to true => no disk write.
    let res = mcp.call_tool(
        "update_element",
        json!({"ref": "Parts::Base", "fields": {"isAbstract": true}}),
    );
    assert_eq!(res.get("written").and_then(|w| w.as_bool()), Some(false), "dry-run does not write");
    assert!(res.get("validationDelta").is_some(), "validation delta returned");
    assert_eq!(dir_hash(&model), before, "disk byte-for-byte unchanged after dry-run");
}

#[test]
fn commit_introducing_new_error_is_refused() {
    let model = fixture_copy();
    let before = dir_hash(&model);
    let mut mcp = Mcp::start(&model);
    mcp.initialize();

    // Point supertype at a non-existent element -> unresolved reference error.
    let res = mcp.call_tool(
        "update_element",
        json!({"ref": "Parts::Derived", "fields": {"supertype": "Parts::DoesNotExist"}, "dry_run": false}),
    );
    assert_eq!(res.get("written").and_then(|w| w.as_bool()), Some(false), "refused on new error: {res}");
    let new_errors = res
        .get("validationDelta")
        .and_then(|d| d.get("newErrors"))
        .and_then(|e| e.as_array())
        .expect("newErrors in delta");
    assert!(!new_errors.is_empty(), "the offending new error is reported");
    assert_eq!(dir_hash(&model), before, "disk unchanged when commit gated");
}

// ---- TC-TRS-MCP-009: write-path confinement ---------------------------------

#[test]
fn create_outside_model_root_is_refused() {
    let model = fixture_copy();
    let before = dir_hash(&model);
    let mut mcp = Mcp::start(&model);
    mcp.initialize();

    // A qname with a parent-directory segment must not let a write escape the model root.
    let res = mcp.call_tool(
        "create_element",
        json!({"qname": "..::Escape", "type": "PartDef", "dry_run": false}),
    );
    assert_eq!(
        res.get("written").and_then(|w| w.as_bool()),
        Some(false),
        "traversal qname refused: {res}"
    );
    assert!(
        !model.parent().unwrap().join("Escape.md").exists(),
        "no file created above the model root"
    );
    assert_eq!(dir_hash(&model), before, "model tree unchanged");
}

#[test]
fn move_outside_model_root_is_refused() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();

    let res = mcp.call_tool(
        "move_element",
        json!({"ref": "Parts::Base", "dest": "..::Escaped", "dry_run": false}),
    );
    assert_eq!(
        res.get("written").and_then(|w| w.as_bool()),
        Some(false),
        "traversal destination refused: {res}"
    );
    assert!(model.join("Parts/Base.md").exists(), "source unchanged");
    assert!(
        !model.parent().unwrap().join("Escaped.md").exists(),
        "no file created above the model root"
    );
}

#[test]
fn clean_commit_is_visible_without_reload() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();

    mcp.call_tool(
        "update_element",
        json!({"ref": "REQ-FX-001", "fields": {"status": "approved"}, "dry_run": false}),
    );
    let got = mcp.call_tool("get_element", json!({"ref": "REQ-FX-001", "detail": true}));
    assert_eq!(
        got.get("frontmatter").and_then(|f| f.get("status")).and_then(|s| s.as_str())
            .or_else(|| got.get("status").and_then(|s| s.as_str())),
        Some("approved"),
        "committed change visible to a subsequent read without explicit reload"
    );
}

// ---- TC-TRS-MCP-009 (symlink escape): write must not follow a symlink out of root ----

#[cfg(unix)]
#[test]
fn create_through_symlinked_dir_is_refused() {
    let model = fixture_copy();
    // `Evil` inside the model points to the model's parent (outside the root).
    let outside = model.parent().unwrap().to_path_buf();
    std::os::unix::fs::symlink(&outside, model.join("Evil")).unwrap();
    let before = dir_hash(&model);

    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool(
        "create_element",
        json!({"qname": "Evil::Pwned", "type": "PartDef", "dry_run": false}),
    );
    assert_eq!(
        res.get("written").and_then(|w| w.as_bool()),
        Some(false),
        "write through a symlinked dir that escapes the root is refused: {res}"
    );
    assert!(!outside.join("Pwned.md").exists(), "no file created at the symlink target outside the root");
    assert_eq!(dir_hash(&model), before, "model tree unchanged");
}
