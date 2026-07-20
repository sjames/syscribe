//! Integration tests for `syscribe lsp` full-reload behavior.
//! Realises TC-TRS-LSP-007.

mod common;
use common::*;

fn symbol_names(res: &serde_json::Value) -> Vec<String> {
    res.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|s| s.get("name").and_then(|n| n.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

const NEW_REQ_CONTENT: &str = "---\ntype: Requirement\nid: REQ-FXLSP-RELOAD\nname: \"Added after startup\"\nstatus: draft\nreqDomain: software\nreqClass: system\n---\n\nAdded after the server started.\n";

// ---- didSave triggers a full reload -----------------------------------------

#[test]
fn did_save_triggers_full_reload() {
    let model = fixture_copy();
    let mut lsp = Lsp::start(&model);
    lsp.initialize();

    let before = lsp.workspace_symbol("REQ-FXLSP-RELOAD");
    assert!(symbol_names(&before).is_empty(), "not present before the file exists");

    let new_path = model.join("Requirements/REQ-FXLSP-RELOAD.md");
    std::fs::write(&new_path, NEW_REQ_CONTENT).unwrap();
    lsp.did_save(&new_path, Some(NEW_REQ_CONTENT));

    // Poll via workspace/symbol (a plain request) until the reload has landed —
    // reload happens synchronously with respect to subsequent requests since both
    // go through the same store lock.
    let after = lsp.workspace_symbol("REQ-FXLSP-RELOAD");
    assert!(symbol_names(&after).contains(&"Added after startup".to_string()), "found after didSave reload: {after:?}");

    lsp.shutdown();
}

// ---- an external file change triggers a full reload -------------------------

#[test]
fn external_change_triggers_full_reload() {
    let model = fixture_copy();
    let mut lsp = Lsp::start(&model);
    lsp.initialize();

    let new_path = model.join("Requirements/REQ-FXLSP-RELOAD.md");
    std::fs::write(&new_path, NEW_REQ_CONTENT).unwrap();
    // No didSave — simulates an edit made outside the editor (e.g. git checkout).
    lsp.did_change_watched_files(&new_path, 1); // FileChangeType::Created

    let after = lsp.workspace_symbol("REQ-FXLSP-RELOAD");
    assert!(
        symbol_names(&after).contains(&"Added after startup".to_string()),
        "found after didChangeWatchedFiles reload: {after:?}"
    );

    lsp.shutdown();
}

// ---- a failed reload preserves prior state -----------------------------------

#[cfg(unix)]
#[test]
fn failed_reload_preserves_prior_state() {
    use std::os::unix::fs::PermissionsExt;

    if unsafe { libc::geteuid() } == 0 {
        eprintln!("skipping failed_reload_preserves_prior_state: running as root, permission bits are ignored");
        return;
    }

    let model = fixture_copy();
    let mut lsp = Lsp::start(&model);
    lsp.initialize();

    let before = lsp.workspace_symbol("Fixture requirement");
    assert!(!symbol_names(&before).is_empty(), "sanity: fixture requirement visible before the failure");

    let req_dir = model.join("Requirements");
    let original_perms = std::fs::metadata(&req_dir).unwrap().permissions();
    std::fs::set_permissions(&req_dir, std::fs::Permissions::from_mode(0o000)).unwrap();

    // Trigger a reload while the model root is transiently unreadable.
    lsp.did_change_watched_files(&model.join("Requirements/REQ-FX-001.md"), 2);

    // Restore permissions before any further filesystem access (including Drop's
    // temp-dir cleanup expectations) so the test doesn't leak an unreadable directory.
    std::fs::set_permissions(&req_dir, original_perms).unwrap();

    let after = lsp.workspace_symbol("Fixture requirement");
    assert!(
        !symbol_names(&after).is_empty(),
        "prior state (with the fixture requirement) preserved across the failed reload: {after:?}"
    );

    lsp.shutdown();
}
