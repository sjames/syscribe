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

// ---- TC-TRS-MCP-021: delete_element -----------------------------------------

#[test]
fn delete_referenced_element_is_blocked() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    // Parts::Derived references Parts::Base via supertype.
    let res = mcp.call_tool("delete_element", json!({"ref": "Parts::Base", "dry_run": false}));
    assert_eq!(res.get("written").and_then(|w| w.as_bool()), Some(false), "referenced delete refused: {res}");
    let blocked: Vec<String> = res
        .get("blockedBy")
        .and_then(|b| b.as_array())
        .map(|a| a.iter().filter_map(|x| x.get("qname").and_then(|q| q.as_str()).map(String::from)).collect())
        .unwrap_or_default();
    assert!(blocked.iter().any(|q| q.contains("Parts::Derived")), "blocking ref from Derived reported; got {blocked:?}");
    assert!(model.join("Parts/Base.md").exists(), "Base still present");
}

#[test]
fn delete_unreferenced_element_succeeds() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    // Nothing references Parts::Derived.
    let res = mcp.call_tool("delete_element", json!({"ref": "Parts::Derived", "dry_run": false}));
    assert_eq!(res.get("written").and_then(|w| w.as_bool()), Some(true), "unreferenced delete commits: {res}");
    assert!(!model.join("Parts/Derived.md").exists(), "Derived removed");
}

#[test]
fn delete_with_force_overrides_references() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool("delete_element", json!({"ref": "Parts::Base", "force": true, "dry_run": false}));
    assert_eq!(res.get("written").and_then(|w| w.as_bool()), Some(true), "force delete commits: {res}");
    assert!(!model.join("Parts/Base.md").exists(), "Base removed under force");
}

// ---- TC-TRS-MCP-022: apply_changes (atomic batch) ---------------------------

#[test]
fn apply_changes_commits_dependent_pair_atomically() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool(
        "apply_changes",
        json!({
            "dry_run": false,
            "operations": [
                {"op": "create", "qname": "Requirements::REQ_BATCH", "type": "Requirement",
                 "fields": {"id": "REQ-BATCH-001", "name": "Batch requirement", "status": "draft", "reqDomain": "software", "reqClass": "system"},
                 "doc": "The system shall support batch creation."},
                {"op": "create", "qname": "Verification::TC_BATCH", "type": "TestCase",
                 "fields": {"id": "TC-BATCH-001", "name": "Batch test", "status": "draft", "testLevel": "L2", "verifies": ["REQ-BATCH-001"]},
                 "doc": "covers REQ-BATCH-001"}
            ]
        }),
    );
    assert_eq!(res.get("written").and_then(|w| w.as_bool()), Some(true), "batch commits: {res}");
    assert!(model.join("Requirements/REQ_BATCH.md").exists(), "first element written");
    assert!(model.join("Verification/TC_BATCH.md").exists(), "second (dependent) element written");
    assert!(res.get("validationDelta").is_some(), "single combined validation delta returned");
}

#[test]
fn apply_changes_rolls_back_fully_on_failure() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    // Second op recreates an existing qname → whole batch must roll back.
    let res = mcp.call_tool(
        "apply_changes",
        json!({
            "dry_run": false,
            "operations": [
                {"op": "create", "qname": "Parts::Good", "type": "PartDef"},
                {"op": "create", "qname": "Parts::Base", "type": "PartDef"}
            ]
        }),
    );
    assert_eq!(res.get("written").and_then(|w| w.as_bool()), Some(false), "failing batch refused: {res}");
    assert!(!model.join("Parts/Good.md").exists(), "first op rolled back — no partial application");
}

// ---- TC-TRS-MCP-023: unified diff preview -----------------------------------

#[test]
fn dry_run_update_returns_unified_diff() {
    let model = fixture_copy();
    let before = dir_hash(&model);
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool(
        "update_element",
        json!({"ref": "REQ-FX-001", "fields": {"status": "approved"}}),
    );
    let diff = res.get("diff").and_then(|d| d.as_str()).expect("diff string present");
    assert!(!diff.is_empty(), "diff is non-empty");
    assert!(diff.contains("approved"), "diff shows the new value as an addition; got {diff}");
    assert_eq!(dir_hash(&model), before, "dry-run leaves disk unchanged");
}

// ---- REQ-TRS-LINKTYPE-002 / -010: link-type errors gate MCP writes ------------

/// A write introducing a `links:` key that is not a declared link type is refused
/// with the E630 finding (naming the declared types) in `newErrors`, so a wrong
/// guess self-corrects over MCP; a declared type commits.
#[test]
fn create_with_undeclared_link_type_is_refused_with_e630() {
    let model = fixture_copy();
    std::fs::write(model.join(".syscribe.toml"), "[linkTypes.informs]\n").unwrap();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();

    let res = mcp.call_tool(
        "create_element",
        json!({"qname": "Parts::Bogus", "type": "PartDef",
               "fields": {"links": {"bogusType": ["Parts::Base"]}}, "dry_run": false}),
    );
    assert_eq!(res.get("written").and_then(|w| w.as_bool()), Some(false), "refused: {res}");
    assert!(!model.join("Parts/Bogus.md").exists(), "nothing written");
    let new_errors = res["validationDelta"]["newErrors"].as_array().cloned().unwrap_or_default();
    let e630 = new_errors
        .iter()
        .find(|e| e["code"] == "E630")
        .unwrap_or_else(|| panic!("E630 in newErrors: {res}"));
    assert!(e630["message"].as_str().unwrap_or("").contains("informs"), "names declared types: {e630}");

    let ok = mcp.call_tool(
        "create_element",
        json!({"qname": "Parts::Good", "type": "PartDef",
               "fields": {"links": {"informs": ["Parts::Base"]}}, "dry_run": false}),
    );
    assert_eq!(ok.get("written").and_then(|w| w.as_bool()), Some(true), "declared type commits: {ok}");
}

// ---- TC-TRS-MCP-008: delta scope and location independence (GH #186, #187) ----

fn codes(delta: &serde_json::Value, key: &str) -> Vec<String> {
    delta[key]
        .as_array()
        .map(|a| a.iter().filter_map(|e| e["code"].as_str().map(String::from)).collect())
        .unwrap_or_default()
}

/// GH #187: an error the commit gate does not cover (E310, a derived requirement with no
/// `breakdownAdr`) is still reported in the delta, flagged `gating: false`; the gate itself is
/// unchanged, so the drafts-stay-creatable rule of REQ-TRS-MCP-008 holds.
#[test]
fn delta_reports_non_gating_errors_flagged_and_does_not_gate_them() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let args = |dry_run: bool| {
        json!({"qname": "Requirements::NoAdr", "type": "Requirement", "dry_run": dry_run,
               "fields": {"id": "REQ-FX-090", "name": "Derived without a breakdown ADR", "status": "draft",
                          "reqDomain": "software", "derivedFrom": ["REQ-FX-001"]},
               "doc": "The system shall do the thing."})
    };

    let dry = mcp.call_tool("create_element", args(true));
    let delta = &dry["validationDelta"];
    let e310 = delta["newErrors"]
        .as_array()
        .and_then(|a| a.iter().find(|e| e["code"] == "E310"))
        .unwrap_or_else(|| panic!("E310 in newErrors: {dry}"));
    assert_eq!(e310["gating"], json!(false), "E310 is reported but does not gate: {e310}");
    assert_eq!(dry["written"], json!(false), "dry run writes nothing");

    let commit = mcp.call_tool("create_element", args(false));
    assert_eq!(commit["written"], json!(true), "a non-gating error does not block the commit: {commit}");
    assert!(codes(&commit["validationDelta"], "newErrors").contains(&"E310".to_string()), "still reported on commit: {commit}");
}

/// GH #187: every error entry carries `gating`; the ones that refuse the commit say `true`.
#[test]
fn gating_errors_are_flagged_gating_true() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = mcp.call_tool(
        "update_element",
        json!({"ref": "Parts::Derived", "fields": {"supertype": "Parts::DoesNotExist"}, "dry_run": true}),
    );
    let eref = res["validationDelta"]["newErrors"]
        .as_array()
        .and_then(|a| a.iter().find(|e| e["code"] == "EREF"))
        .unwrap_or_else(|| panic!("EREF in newErrors: {res}"));
    assert_eq!(eref["gating"], json!(true), "{eref}");
}

/// GH #187: fixing a non-gating error shows up under `resolvedErrors`.
#[test]
fn resolving_a_non_gating_error_is_reported() {
    let model = fixture_copy();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let created = mcp.call_tool(
        "create_element",
        json!({"qname": "Requirements::NoAdr", "type": "Requirement", "dry_run": false,
               "fields": {"id": "REQ-FX-090", "name": "Derived without a breakdown ADR", "status": "draft",
                          "reqDomain": "software", "derivedFrom": ["REQ-FX-001"]},
               "doc": "The system shall do the thing."}),
    );
    assert_eq!(created["written"], json!(true), "{created}");
    let fix = mcp.call_tool(
        "update_element",
        json!({"ref": "REQ-FX-090", "fields": {"breakdownAdr": "Decisions::ADR-FX-001"}, "dry_run": true}),
    );
    assert!(codes(&fix["validationDelta"], "resolvedErrors").contains(&"E310".to_string()), "{fix}");
}

fn set_plantuml_style(model: &std::path::Path, style: &str) {
    let mut cfg = std::fs::read_to_string(model.join(".syscribe.toml")).unwrap();
    cfg.push_str(&format!("\n[plantuml]\nstyle_file = \"{style}\"\n"));
    std::fs::write(model.join(".syscribe.toml"), cfg).unwrap();
}

fn create_part_dry_run(mcp: &mut Mcp) -> serde_json::Value {
    mcp.call_tool("create_element", json!({"qname": "Parts::Extra", "type": "PartDef", "dry_run": true}))
}

/// GH #186: a config path that is relative to the model root but points outside it must
/// resolve the same way when the candidate is validated, so it yields no spurious W415.
#[test]
fn relative_path_outside_model_root_gives_no_spurious_finding() {
    let model = fixture_copy();
    std::fs::write(model.parent().unwrap().join("style.iuml"), "skinparam monochrome true\n").unwrap();
    set_plantuml_style(&model, "../style.iuml");
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = create_part_dry_run(&mut mcp);
    let delta = &res["validationDelta"];
    for key in ["newWarnings", "resolvedWarnings"] {
        assert!(!codes(delta, key).contains(&"W415".to_string()), "no W415 in {key}: {res}");
    }
}

/// GH #186: a finding that exists both before and after (here a genuinely missing style file)
/// must cancel out — its message names the model root, which differs between the real tree
/// and the candidate copy, so the messages must be normalised before the delta is taken.
#[test]
fn preexisting_finding_is_not_reported_as_new_and_resolved() {
    let model = fixture_copy();
    set_plantuml_style(&model, "../does-not-exist.iuml");
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    let res = create_part_dry_run(&mut mcp);
    let delta = &res["validationDelta"];
    for key in ["newWarnings", "resolvedWarnings"] {
        assert!(!codes(delta, key).contains(&"W415".to_string()), "pre-existing W415 must cancel in {key}: {res}");
    }
}

fn leftover_candidates(dir: &std::path::Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with(".syscribe-mcp-cand-"))
        .collect()
}

/// GH #186: the sibling staging directory is removed after dry-run, commit and refusal alike.
#[test]
fn candidate_staging_dir_is_cleaned_up() {
    let model = fixture_copy();
    let parent = model.parent().unwrap().to_path_buf();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    create_part_dry_run(&mut mcp);
    mcp.call_tool("create_element", json!({"qname": "Parts::Committed", "type": "PartDef", "dry_run": false}));
    mcp.call_tool(
        "update_element",
        json!({"ref": "Parts::Derived", "fields": {"supertype": "Parts::DoesNotExist"}, "dry_run": false}),
    );
    assert_eq!(leftover_candidates(&parent), Vec::<String>::new(), "no staging directory left behind");
}

/// GH #186: when the model root's parent is not writable the guard still works, staging in the
/// system temp dir instead (and leaves nothing behind in the parent).
#[cfg(unix)]
#[test]
fn unwritable_parent_falls_back_to_temp_staging() {
    use std::os::unix::fs::PermissionsExt;
    let model = fixture_copy();
    let parent = model.parent().unwrap().to_path_buf();
    let mut mcp = Mcp::start(&model);
    mcp.initialize();
    std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o555)).unwrap();
    let res = create_part_dry_run(&mut mcp);
    std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(res.get("validationDelta").is_some(), "guard still produces a delta: {res}");
    assert_eq!(res["written"], json!(false));
    assert_eq!(leftover_candidates(&parent), Vec::<String>::new());
}
