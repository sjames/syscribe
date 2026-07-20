//! Integration tests for `syscribe lsp` completion.
//! Realises TC-TRS-LSP-008 (field-aware) and TC-TRS-LSP-009 (enum-aware).

mod common;
use common::*;

fn labels(res: &serde_json::Value) -> Vec<String> {
    let items = res
        .as_array()
        .cloned()
        .or_else(|| res.get("items").and_then(|i| i.as_array()).cloned())
        .unwrap_or_default();
    items.into_iter().filter_map(|i| i.get("label").and_then(|l| l.as_str()).map(String::from)).collect()
}

// ---- TC-TRS-LSP-008: field-aware completion ---------------------------------

#[test]
fn completion_in_verifies_field_offers_only_requirement_ids() {
    let model = fixture_copy();
    let tc_path = model.join("Verification/TC-FX-001.md");

    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    // Line 7 is `  - REQ-FX-001` under `verifies:`.
    let res = lsp.completion(&tc_path, 7, 8);
    let labels = labels(&res);
    assert!(!labels.is_empty(), "some candidates offered");
    assert!(labels.iter().all(|l| l.starts_with("REQ-")), "every candidate is a Requirement id: {labels:?}");

    lsp.shutdown();
}

#[test]
fn completion_in_breakdownadr_field_offers_only_adr_ids() {
    let model = fixture_copy();
    let req_path = model.join("Requirements/REQ-FXCHILD-001.md");

    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    // Line 8 is `breakdownAdr: Decisions::ADR-FX-001`.
    let res = lsp.completion(&req_path, 8, 20);
    let labels = labels(&res);
    assert!(!labels.is_empty(), "some candidates offered");
    assert!(labels.iter().all(|l| l.starts_with("ADR-")), "every candidate is an ADR id: {labels:?}");

    lsp.shutdown();
}

// ---- TC-TRS-LSP-009: enum-aware completion ----------------------------------

#[test]
fn completion_on_status_field_offers_the_requirement_status_domain() {
    let model = fixture_copy();
    let req_path = model.join("Requirements/REQ-FX-001.md");

    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    // Line 4 is `status: draft`.
    let res = lsp.completion(&req_path, 4, 10);
    let labels = labels(&res);
    for expected in ["draft", "review", "approved", "implemented", "verified"] {
        assert!(labels.contains(&expected.to_string()), "{expected} offered: {labels:?}");
    }

    lsp.shutdown();
}

#[test]
fn completion_on_type_field_offers_known_element_types() {
    let model = fixture_copy();
    let req_path = model.join("Requirements/REQ-FX-001.md");

    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    // Line 1 is `type: Requirement`.
    let res = lsp.completion(&req_path, 1, 10);
    let labels = labels(&res);
    for expected in ["Requirement", "TestCase", "ADR"] {
        assert!(labels.contains(&expected.to_string()), "{expected} offered: {labels:?}");
    }

    lsp.shutdown();
}
