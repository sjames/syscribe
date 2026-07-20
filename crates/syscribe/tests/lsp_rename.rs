//! Integration tests for `syscribe lsp` prepareRename/rename.
//! Realises TC-TRS-LSP-010, TC-TRS-LSP-011, TC-TRS-LSP-012.

mod common;
use common::*;

// ---- TC-TRS-LSP-010: prepareRename -------------------------------------------

#[test]
fn prepare_rename_on_a_stable_id_succeeds() {
    let model = fixture_copy();
    let req_path = model.join("Requirements/REQ-FX-001.md");

    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    // Line 2 is `id: REQ-FX-001`.
    let res = lsp.prepare_rename(&req_path, 2, 8);
    let placeholder = res.get("placeholder").and_then(|p| p.as_str()).expect("placeholder present");
    assert_eq!(placeholder, "REQ-FX-001");

    lsp.shutdown();
}

#[test]
fn prepare_rename_on_a_name_identified_element_is_refused() {
    let model = fixture_copy();
    let derived_path = model.join("Parts/Derived.md");

    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    // Line 3 is `supertype: Parts::Base` — resolves to the name-identified PartDef Base.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| lsp.prepare_rename(&derived_path, 3, 15)));
    assert!(result.is_err(), "name-identified rename target refused with an LSP error");
}

// ---- TC-TRS-LSP-011: rename computes a WorkspaceEdit -------------------------

#[test]
fn rename_edits_the_requirement_and_its_verifying_testcase() {
    let model = fixture_copy();
    let req_path = model.join("Requirements/REQ-FX-001.md");
    let tc_path = model.join("Verification/TC-FX-001.md");

    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    // Line 2 is `id: REQ-FX-001`.
    let res = lsp.rename(&req_path, 2, 8, "REQ-FX-777");
    let changes = res.get("changes").and_then(|c| c.as_object()).expect("changes map present");
    let keys: Vec<&String> = changes.keys().collect();
    assert!(keys.iter().any(|k| k.ends_with("REQ-FX-001.md")), "edits the requirement's own file: {keys:?}");
    assert!(keys.iter().any(|k| k.ends_with("TC-FX-001.md")), "edits the verifying TestCase's file: {keys:?}");

    // The server must not have written anything to disk itself.
    assert!(std::fs::read_to_string(&req_path).unwrap().contains("REQ-FX-001"), "disk unchanged by the server");
    assert!(std::fs::read_to_string(&tc_path).unwrap().contains("REQ-FX-001"), "disk unchanged by the server");

    lsp.shutdown();
}

#[test]
fn rename_with_a_malformed_new_id_is_refused() {
    let model = fixture_copy();
    let req_path = model.join("Requirements/REQ-FX-001.md");

    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| lsp.rename(&req_path, 2, 8, "not-a-valid-id")));
    assert!(result.is_err(), "malformed new id refused with an LSP error");
}

// ---- TC-TRS-LSP-012: rename safety gate --------------------------------------

#[test]
fn rename_to_a_colliding_id_is_refused() {
    let model = fixture_copy();
    let req_path = model.join("Requirements/REQ-FX-001.md");

    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    // REQ-FX-003 already exists in the fixture.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| lsp.rename(&req_path, 2, 8, "REQ-FX-003")));
    assert!(result.is_err(), "colliding id refused with an LSP error");
}
