//! MCP `binding_times` tool (REQ-TRS-BT-005, realised by TC-TRS-BT-005): the
//! effective binding-time vocabulary, same payload as `binding-times --json`.

mod common;
use common::*;
use serde_json::json;
use std::path::{Path, PathBuf};

/// A throwaway model root holding a `.syscribe.toml` and one feature.
fn model_with(toml: Option<&str>) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("syscribe_bt_mcp_{}_{}", std::process::id(), toml.is_some()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("Features")).unwrap();
    if let Some(t) = toml {
        std::fs::write(dir.join(".syscribe.toml"), t).unwrap();
    }
    std::fs::write(
        dir.join("Features/Motor.md"),
        "---\ntype: FeatureDef\nid: FEAT-FX-BT1\nname: Motor\ngroupKind: optional\n---\nA feature.\n",
    )
    .unwrap();
    dir
}

fn names(res: &serde_json::Value) -> Vec<String> {
    res["bindingTimes"]
        .as_array()
        .unwrap_or_else(|| panic!("no bindingTimes array in {res}"))
        .iter()
        .map(|s| s["name"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn binding_times_lists_the_builtin_vocabulary() {
    let model = model_with(None);
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("binding_times", json!({}));
    assert_eq!(names(&res), ["compile", "load", "runtime"]);
    assert_eq!(res["bindingTimes"][2]["builtin"], true);
    drop(mcp);
    let _ = std::fs::remove_dir_all(Path::new(&model));
}

#[test]
fn binding_times_reflects_a_refined_vocabulary_in_order() {
    let model = model_with(Some(
        "[bindingTimes.deploy]\nbinder = \"installer\"\nafter = \"compile\"\nbefore = \"load\"\n",
    ));
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("binding_times", json!({}));
    assert_eq!(names(&res), ["compile", "deploy", "load", "runtime"]);
    assert_eq!(res["bindingTimes"][1]["binder"], "installer");
    assert_eq!(res["bindingTimes"][1]["builtin"], false);
    drop(mcp);
    let _ = std::fs::remove_dir_all(Path::new(&model));
}
