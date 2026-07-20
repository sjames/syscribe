//! Integration tests for `syscribe lsp` navigation capabilities.
//! Realises TC-TRS-LSP-003 (definition), TC-TRS-LSP-004 (references),
//! TC-TRS-LSP-005 (hover), TC-TRS-LSP-006 (workspace/symbol).

mod common;
use common::*;

// ---- TC-TRS-LSP-003: go to definition ---------------------------------------

#[test]
fn definition_resolves_verifies_reference_to_target_file() {
    let model = fixture_copy();
    let tc_path = model.join("Verification/TC-FX-001.md");
    let req_path = model.join("Requirements/REQ-FX-001.md");

    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    // Line 7 is `  - REQ-FX-001` (the verifies list entry).
    let res = lsp.definition(&tc_path, 7, 8);
    let uri = res.get("uri").and_then(|u| u.as_str()).expect("definition location uri");
    assert_eq!(uri, file_uri(&req_path), "resolves to the requirement's own file");

    lsp.shutdown();
}

#[test]
fn definition_on_non_reference_position_returns_null() {
    let model = fixture_copy();
    let tc_path = model.join("Verification/TC-FX-001.md");

    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    // Line 0 is the opening `---` frontmatter delimiter — not a reference.
    let res = lsp.definition(&tc_path, 0, 1);
    assert!(res.is_null(), "no definition at a non-reference position: {res:?}");

    lsp.shutdown();
}

// ---- TC-TRS-LSP-004: find references ----------------------------------------

#[test]
fn references_on_a_requirement_finds_its_verifying_testcase() {
    let model = fixture_copy();
    let req_path = model.join("Requirements/REQ-FX-001.md");
    let tc_path = model.join("Verification/TC-FX-001.md");

    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    // Line 2 is `id: REQ-FX-001`.
    let res = lsp.references(&req_path, 2, 8);
    let locations = res.as_array().expect("references array");
    let uris: Vec<&str> = locations.iter().filter_map(|l| l.get("uri").and_then(|u| u.as_str())).collect();
    assert!(uris.contains(&file_uri(&tc_path).as_str()), "verifying TestCase found: {uris:?}");
    // The client's references() sets includeDeclaration: true, so the declaration
    // itself is included alongside the referencer.
    assert!(uris.contains(&file_uri(&req_path).as_str()), "declaration included per includeDeclaration: true: {uris:?}");

    lsp.shutdown();
}

#[test]
fn references_on_an_unreferenced_element_is_empty() {
    let model = fixture_copy();
    let req_path = model.join("Requirements/REQ-FX-003.md");

    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    // Line 2 is `id: REQ-FX-003`.
    let res = lsp.references(&req_path, 2, 8);
    let locations = res.as_array().expect("references array, not an error");
    let uris: Vec<&str> = locations.iter().filter_map(|l| l.get("uri").and_then(|u| u.as_str())).collect();
    assert!(!uris.iter().any(|u| !u.ends_with("REQ-FX-003.md")), "no other-file references: {uris:?}");

    lsp.shutdown();
}

// ---- TC-TRS-LSP-005: hover ---------------------------------------------------

#[test]
fn hover_over_a_resolvable_reference_shows_the_target_summary() {
    let model = fixture_copy();
    let tc_path = model.join("Verification/TC-FX-001.md");

    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    let res = lsp.hover(&tc_path, 7, 8);
    let value = res
        .get("contents")
        .and_then(|c| c.get("value"))
        .and_then(|v| v.as_str())
        .expect("hover markup value");
    assert!(value.contains("REQ-FX-001"), "hover shows the target's id: {value}");
    assert!(value.contains("Requirement"), "hover shows the target's type: {value}");

    lsp.shutdown();
}

#[test]
fn hover_over_free_text_returns_null() {
    let model = fixture_copy();
    let req_path = model.join("Requirements/REQ-FX-001.md");

    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    // Line 3 is `name: "Fixture requirement for MCP tests"` — free-text label, not a
    // resolvable id/qname.
    let res = lsp.hover(&req_path, 3, 8);
    assert!(res.is_null(), "no hover over free text: {res:?}");

    lsp.shutdown();
}

// ---- TC-TRS-LSP-006: workspace/symbol ---------------------------------------

#[test]
fn workspace_symbol_finds_by_stable_id() {
    let model = fixture_copy();
    let req_path = model.join("Requirements/REQ-FX-001.md");

    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    let res = lsp.workspace_symbol("REQ-FX-001");
    let symbols = res.as_array().expect("symbol array");
    let uris: Vec<&str> = symbols.iter().filter_map(|s| s.get("location")?.get("uri")?.as_str()).collect();
    assert!(uris.contains(&file_uri(&req_path).as_str()), "found by id: {uris:?}");

    lsp.shutdown();
}

#[test]
fn workspace_symbol_non_matching_query_is_empty_not_an_error() {
    let model = fixture_copy();
    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    let res = lsp.workspace_symbol("zzz-no-such-element-in-the-fixture-zzz");
    let symbols = res.as_array().expect("symbol array, not an error");
    assert!(symbols.is_empty(), "no matches: {symbols:?}");

    lsp.shutdown();
}
