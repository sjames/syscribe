//! `edit_feature` (`REQ-TRS-FMED-004`): the MCP counterpart of the browser's Edit mode. One
//! semantic feature-model edit through the guarded write, with the effect on the model's
//! validity (`featureDelta`), a hold for an edit that makes it worse, and an `undo`.

mod common;
use common::*;
use serde_json::json;
use std::path::{Path, PathBuf};

fn write(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

/// Car (mandatory) > Engine (mandatory, alternative: Petrol, Electric), Charger; Electric requires Charger.
fn model() -> PathBuf {
    let r = std::env::temp_dir().join(format!("syscribe-mcpfe-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::create_dir_all(&r).unwrap();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "F/_index.md", "---\ntype: Package\nname: F\n---\n");
    write(&r, "F/Car.md", "---\ntype: FeatureDef\nid: FEAT-CAR\nname: Car\nmandatory: true\n---\n");
    write(&r, "F/Car/Engine.md", "---\ntype: FeatureDef\nid: FEAT-ENGINE\nname: Engine\nmandatory: true\ngroupKind: alternative\n---\n");
    write(&r, "F/Car/Engine/Petrol.md", "---\ntype: FeatureDef\nid: FEAT-PETROL\nname: Petrol\n---\n");
    write(&r, "F/Car/Engine/Electric.md", "---\ntype: FeatureDef\nid: FEAT-ELECTRIC\nname: Electric\nrequires: [FEAT-CHARGER]\n---\n");
    write(&r, "F/Car/Charger.md", "---\ntype: FeatureDef\nid: FEAT-CHARGER\nname: Charger\n---\n");
    r
}

#[test]
fn a_dry_run_reports_the_effect_and_writes_nothing() {
    let r = model();
    let before = dir_hash(&r);
    let mut mcp = Mcp::start(&r);
    mcp.initialize();
    let res = mcp.call_tool("edit_feature", json!({"edit": {"op": "add", "parent": "F::Car", "name": "Sunroof"}}));
    assert_eq!(res["written"], false, "dry_run defaults to true: {res}");
    assert_eq!(res["featureDelta"]["worsens"], false, "{res}");
    assert_eq!(res["featureDelta"]["countsAfter"]["features"], 6);
    assert_eq!(dir_hash(&r), before);
}

#[test]
fn a_commit_writes_reloads_and_returns_the_undo() {
    let r = model();
    let mut mcp = Mcp::start(&r);
    mcp.initialize();
    let res = mcp.call_tool("edit_feature", json!({"edit": {"op": "add", "parent": "F::Car", "name": "Sunroof"}, "dry_run": false}));
    assert_eq!(res["written"], true, "{res}");
    assert_eq!(res["feature"], "F::Car::Sunroof");
    assert!(r.join("F/Car/Sunroof.md").exists());
    // The live store reloaded.
    let got = mcp.call_tool("get_element", json!({"ref": "F::Car::Sunroof"}));
    assert_eq!(got["qname"], "F::Car::Sunroof");
    let undone = mcp.call_tool("edit_feature", json!({"edit": res["undo"].clone(), "dry_run": false}));
    assert_eq!(undone["written"], true, "{undone}");
    assert!(!r.join("F/Car/Sunroof.md").exists());
}

#[test]
fn an_edit_that_makes_the_model_worse_is_held_until_accepted() {
    let r = model();
    let mut mcp = Mcp::start(&r);
    mcp.initialize();
    let op = json!({"op": "addConstraint", "feature": "FEAT-CHARGER", "kind": "excludes", "target": "FEAT-ELECTRIC"});
    let held = mcp.call_tool("edit_feature", json!({"edit": op.clone(), "dry_run": false}));
    assert_eq!(held["written"], false, "{held}");
    assert_eq!(held["needsConfirmation"], true);
    assert_eq!(held["featureDelta"]["newDead"], json!(["F::Car::Engine::Electric"]));
    assert!(!std::fs::read_to_string(r.join("F/Car/Charger.md")).unwrap().contains("excludes"));
    let done = mcp.call_tool("edit_feature", json!({"edit": op, "dry_run": false, "accept_worse": true}));
    assert_eq!(done["written"], true, "{done}");
    assert!(std::fs::read_to_string(r.join("F/Car/Charger.md")).unwrap().contains("excludes"));
}

#[test]
fn parameters_are_edited_and_a_malformed_edit_is_a_tool_error() {
    let r = model();
    let mut mcp = Mcp::start(&r);
    mcp.initialize();
    let res = mcp.call_tool("edit_feature", json!({"edit": {"op": "setParameter", "feature": "FEAT-ELECTRIC", "parameter": {"name": "kw", "type": "ScalarValues::Real", "range": "50..=300"}}, "dry_run": false}));
    assert_eq!(res["written"], true, "{res}");
    assert!(std::fs::read_to_string(r.join("F/Car/Engine/Electric.md")).unwrap().contains("kw"));
    let removed = mcp.call_tool("edit_feature", json!({"edit": {"op": "removeParameter", "feature": "FEAT-ELECTRIC", "name": "kw"}, "dry_run": false}));
    assert_eq!(removed["written"], true, "{removed}");
    let raw = mcp.call_tool_raw("edit_feature", json!({"edit": {"op": "explode"}}));
    assert_eq!(raw["isError"], true, "{raw}");
    let refused = mcp.call_tool("edit_feature", json!({"edit": {"op": "remove", "feature": "F::Car::Engine"}, "dry_run": false}));
    assert_eq!(refused["written"], false);
    assert!(refused["reason"].as_str().unwrap().contains("below it"), "{refused}");
}

#[test]
fn the_tool_is_hidden_and_rejected_in_read_only_mode() {
    let r = model();
    let mut mcp = Mcp::start_with_args(&r, &["--read-only"]);
    mcp.initialize();
    let tools = mcp.tools_list();
    let names: Vec<&str> = tools["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(!names.contains(&"edit_feature"), "{names:?}");
    let raw = mcp.call_tool_raw("edit_feature", json!({"edit": {"op": "add", "name": "X"}, "dry_run": false}));
    assert!(raw["isError"] == true || raw.get("error").is_some(), "{raw}");
    assert!(!r.join("F/X.md").exists());
}
