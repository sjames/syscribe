//! Integration tests for `syscribe lsp` diagnostics publishing.
//! Realises TC-TRS-LSP-002.

mod common;
use common::*;

fn diag_codes(params: &serde_json::Value) -> Vec<String> {
    params
        .get("diagnostics")
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|d| d.get("code").and_then(|c| c.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

// ---- opening a file with a validator error publishes a diagnostic ----------

#[test]
fn opening_file_with_validator_error_publishes_diagnostic() {
    let model = fixture_copy();
    let bad = model.join("Requirements/REQ-FXLSP-001.md");
    let content = "---\n\
type: Requirement\n\
id: REQ-FXLSP-001\n\
title: \"Bad label\"\n\
name: \"Also has a name\"\n\
status: draft\n\
reqDomain: software\n\
reqClass: system\n\
---\n\
\n\
Body text.\n";
    std::fs::write(&bad, content).unwrap();

    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    lsp.did_open(&bad, "markdown", content);

    let params = lsp.wait_for_notification("textDocument/publishDiagnostics");
    let uri = params.get("uri").and_then(|u| u.as_str()).expect("uri present");
    assert!(uri.ends_with("REQ-FXLSP-001.md"), "diagnostics for the opened file, got {uri}");

    let diags = params.get("diagnostics").and_then(|d| d.as_array()).expect("diagnostics array");
    let e025 = diags
        .iter()
        .find(|d| d.get("code").and_then(|c| c.as_str()) == Some("E025"))
        .unwrap_or_else(|| panic!("E025 present among {diags:?}"));
    assert_eq!(e025.get("severity").and_then(|s| s.as_u64()), Some(1), "Error maps to LSP severity 1");

    // Best-effort range (REQ-TRS-LSP-002): spans the frontmatter block. The closing
    // `---` is line 8 (0-indexed) in the fixture content above.
    let range = e025.get("range").expect("range present");
    assert_eq!(range["start"]["line"].as_u64(), Some(0));
    assert!(range["end"]["line"].as_u64().unwrap() >= 8, "range end reaches the closing '---': {range}");

    lsp.shutdown();
}

// ---- an external edit introduces a finding located in a different file ----

#[test]
fn external_change_republishes_diagnostics_for_the_referencing_file() {
    let model = fixture_copy();
    let tc_path = model.join("Verification/TC-FX-001.md");
    let req_path = model.join("Requirements/REQ-FX-001.md");

    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    lsp.did_open(&tc_path, "markdown", &std::fs::read_to_string(&tc_path).unwrap());

    let initial = lsp.wait_for_notification("textDocument/publishDiagnostics");
    assert!(!diag_codes(&initial).contains(&"E102".to_string()), "no E102 before the reference is broken");

    // Break the reference from disk, out from under the editor (file B).
    let content = std::fs::read_to_string(&req_path).unwrap();
    std::fs::write(&req_path, content.replace("id: REQ-FX-001", "id: REQ-FX-999")).unwrap();
    lsp.did_change_watched_files(&req_path, 2); // FileChangeType::Changed

    let params = lsp.wait_for_notification("textDocument/publishDiagnostics");
    let uri = params.get("uri").and_then(|u| u.as_str()).expect("uri present");
    assert!(uri.ends_with("TC-FX-001.md"), "diagnostics republished for file A (the referencer), got {uri}");
    assert!(diag_codes(&params).contains(&"E102".to_string()), "E102 (unresolved verifies) now present: {params:?}");

    lsp.shutdown();
}

// ---- fixing a finding clears its diagnostic ---------------------------------

#[test]
fn saving_a_fix_clears_the_diagnostic() {
    let model = fixture_copy();
    let bad = model.join("Requirements/REQ-FXLSP-002.md");
    let bad_content = "---\ntype: Requirement\nid: REQ-FXLSP-002\ntitle: \"Bad\"\nname: \"N\"\nstatus: draft\nreqDomain: software\nreqClass: system\n---\n\nBody.\n";
    std::fs::write(&bad, bad_content).unwrap();

    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    lsp.did_open(&bad, "markdown", bad_content);
    let first = lsp.wait_for_notification("textDocument/publishDiagnostics");
    assert!(diag_codes(&first).contains(&"E025".to_string()), "E025 present before the fix");

    let fixed_content = "---\ntype: Requirement\nid: REQ-FXLSP-002\nname: \"N\"\nstatus: draft\nreqDomain: software\nreqClass: system\n---\n\nBody.\n";
    std::fs::write(&bad, fixed_content).unwrap();
    lsp.did_save(&bad, Some(fixed_content));

    let second = lsp.wait_for_notification("textDocument/publishDiagnostics");
    assert!(!diag_codes(&second).contains(&"E025".to_string()), "E025 cleared after the fix: {second:?}");

    lsp.shutdown();
}

// ---- diagnostics do not change on didChange alone ---------------------------

#[test]
fn did_change_alone_does_not_republish_diagnostics() {
    let model = fixture_copy();
    let path = model.join("Requirements/REQ-FX-001.md");
    let original = std::fs::read_to_string(&path).unwrap();

    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    lsp.did_open(&path, "markdown", &original);
    let initial = lsp.wait_for_notification("textDocument/publishDiagnostics");
    assert!(!diag_codes(&initial).contains(&"E025".to_string()), "clean before the unsaved edit");

    // Send an in-buffer edit that introduces E025 — but never save it.
    let bad = original.replacen("id: REQ-FX-001", "id: REQ-FX-001\ntitle: \"bad\"", 1);
    lsp.did_change(&path, 2, &bad);

    // Synchronize on a request/response round-trip; any notification emitted before
    // the response lands in `lsp.notifications`.
    lsp.workspace_symbol("REQ-FX-001");
    let saw_new_error = lsp.notifications.iter().any(|n| {
        n.get("method").and_then(|m| m.as_str()) == Some("textDocument/publishDiagnostics")
            && diag_codes(n.get("params").unwrap_or(&serde_json::Value::Null)).contains(&"E025".to_string())
    });
    assert!(!saw_new_error, "no E025 from didChange alone — v1 validates saved disk state only");

    lsp.shutdown();
}
