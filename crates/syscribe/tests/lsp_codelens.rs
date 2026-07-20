//! Integration tests for `syscribe lsp` codeLens.
//! Realises TC-TRS-LSP-013.

mod common;
use common::*;

#[test]
fn verified_requirement_gets_a_display_only_lens() {
    let model = fixture_copy();
    let req_path = model.join("Requirements/REQ-FX-001.md");

    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    let res = lsp.code_lens(&req_path);
    let lenses = res.as_array().expect("codeLens array");
    assert!(!lenses.is_empty(), "at least one lens for a verified requirement: {lenses:?}");
    let titles: Vec<&str> = lenses.iter().filter_map(|l| l.get("command")?.get("title")?.as_str()).collect();
    assert!(titles.iter().any(|t| t.contains("verifiedBy")), "a lens mentions verifiedBy: {titles:?}");
    assert!(
        lenses.iter().all(|l| l.get("command").and_then(|c| c.get("command")).and_then(|c| c.as_str()) == Some("")),
        "no click action (empty command id): {lenses:?}"
    );

    lsp.shutdown();
}
