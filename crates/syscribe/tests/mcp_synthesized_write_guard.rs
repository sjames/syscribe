//! Regression coverage (REQ-TRS-FM-005 corner-case review) for a serious
//! write-path gap: `update_element`/`delete_element`/`move_element` resolve a
//! qname to a `file_path` and then rewrite/remove/relocate that *whole file*.
//! Pointed at an element synthesized from a shared sheet (an FMEA/TARA entry,
//! or — the case exercised here — a `FeatureModel` sheet's `featureTree:`
//! entry), that file is the *sheet*, not a file of that element's own:
//!
//! - `update_element` would patch the sheet's own top-level frontmatter
//!   instead of the buried list entry (silently doing nothing useful).
//! - `delete_element` would delete the sheet file — every sibling feature
//!   lost, not just the targeted one.
//! - `move_element` would relocate the whole sheet file to a path derived
//!   from the *one feature's* new name.
//!
//! All three must now refuse instead. A normal, per-file `FeatureDef` in the
//! same model (`Features::Legacy::SafeMode`) is the positive control, proving
//! the guard doesn't overreach onto genuinely 1:1 elements.

mod common;
use common::*;
use serde_json::json;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

fn temp_model() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "syscribe-mcp-synth-guard-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let features = dir.join("Features");
    std::fs::create_dir_all(&features).unwrap();
    std::fs::write(
        features.join("_index.md"),
        r#"---
type: FeatureModel
name: Features
featureTree:
  - name: Platform
    id: FEAT-PLATFORM-001
    mandatory: true
    groupKind: alternative
  - name: Platform.CortexM
    id: FEAT-CORTEXM-001
    groupKind: optional
---
"#,
    )
    .unwrap();
    std::fs::write(
        features.join("SafeMode.md"),
        "---\ntype: FeatureDef\nid: FEAT-SAFEMODE-001\nname: SafeMode\ngroupKind: optional\n---\n",
    )
    .unwrap();
    dir
}

fn is_error(res: &serde_json::Value) -> bool {
    res.get("isError").and_then(|e| e.as_bool()) == Some(true)
        || res.get("written").and_then(|w| w.as_bool()) == Some(false)
}

fn text_of(res: &serde_json::Value) -> String {
    res.get("content")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .and_then(|item| item.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or_default()
        .to_string()
}

#[test]
fn update_element_refuses_on_a_synthesized_feature() {
    let model = temp_model();
    let sheet = model.join("Features/_index.md");
    let before = std::fs::read(&sheet).unwrap();

    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool_raw(
        "update_element",
        json!({"ref": "Features::Platform::CortexM", "fields": {"mandatory": true}, "dry_run": false}),
    );
    assert!(is_error(&res), "update on a synthesized feature must refuse: {res}");
    assert!(text_of(&res).contains("synthesized"), "refusal names the reason: {res}");

    let after = std::fs::read(&sheet).unwrap();
    assert_eq!(before, after, "sheet file must be left byte-for-byte unchanged");
}

#[test]
fn delete_element_refuses_on_a_synthesized_feature() {
    let model = temp_model();
    let sheet = model.join("Features/_index.md");

    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool_raw(
        "delete_element",
        json!({"ref": "Features::Platform::CortexM", "dry_run": false}),
    );
    assert!(is_error(&res), "delete on a synthesized feature must refuse: {res}");
    assert!(text_of(&res).contains("synthesized"), "refusal names the reason: {res}");
    assert!(sheet.exists(), "sheet file (every sibling feature) must survive");
}

#[test]
fn move_element_refuses_on_a_synthesized_feature() {
    let model = temp_model();
    let sheet = model.join("Features/_index.md");
    let before = std::fs::read(&sheet).unwrap();

    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    // Unlike update_element/delete_element's early tool_error refusal, this
    // one surfaces from inside mv::move_element via the normal guarded-write
    // path — a protocol-level success (isError: false) carrying
    // written:false and a `reason`, not an MCP tool error.
    let res = mcp.call_tool(
        "move_element",
        json!({"ref": "Features::Platform::CortexM", "dest": "Features::Platform::Renamed", "dry_run": false}),
    );
    assert_eq!(res.get("written").and_then(|w| w.as_bool()), Some(false), "move on a synthesized feature must refuse: {res}");
    assert!(
        res.get("reason").and_then(|r| r.as_str()).unwrap_or_default().contains("synthesized"),
        "refusal names the reason: {res}"
    );

    assert!(sheet.exists(), "sheet must not be relocated");
    assert!(!model.join("Features/Platform/Renamed.md").exists(), "no file created at the derived destination");
    let after = std::fs::read(&sheet).unwrap();
    assert_eq!(before, after, "sheet file must be left byte-for-byte unchanged");
}

/// Positive control: a genuine per-file `FeatureDef` in the *same* model is
/// unaffected by the guard — proves it targets synthesis, not `FeatureDef`
/// as a type.
#[test]
fn update_element_still_works_on_a_genuine_per_file_feature() {
    let model = temp_model();

    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool(
        "update_element",
        json!({"ref": "Features::SafeMode", "fields": {"mandatory": true}, "dry_run": false}),
    );
    assert_eq!(res.get("written").and_then(|w| w.as_bool()), Some(true), "committed: {res}");
    let content = std::fs::read_to_string(model.join("Features/SafeMode.md")).unwrap();
    assert!(content.contains("mandatory: true"), "field actually updated: {content}");
}

/// Also unaffected: an ordinary non-PLE element (no explosion mechanism in
/// play at all) — the baseline the guard must never touch.
#[test]
fn delete_element_still_works_on_an_ordinary_element() {
    let model = temp_model();
    std::fs::write(
        model.join("Standalone.md"),
        "---\ntype: PartDef\nname: Standalone\n---\n",
    )
    .unwrap();

    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("delete_element", json!({"ref": "Standalone", "dry_run": false}));
    assert_eq!(res.get("written").and_then(|w| w.as_bool()), Some(true), "committed: {res}");
    assert!(!model.join("Standalone.md").exists());
}
