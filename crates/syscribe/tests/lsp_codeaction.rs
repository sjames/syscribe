//! Integration tests for `syscribe lsp` codeAction (+ its executeCommand backend).
//! Realises TC-TRS-LSP-014, TC-TRS-LSP-015, TC-TRS-LSP-016.

mod common;
use common::*;

// ---- TC-TRS-LSP-014: E310 quick-fix ------------------------------------------

#[test]
fn e310_gets_one_breakdownadr_action_per_accepted_adr() {
    let model = fixture_copy();
    let path = model.join("Requirements/REQ-FXLSP-E310.md");
    std::fs::write(
        &path,
        "---\ntype: Requirement\nid: REQ-FXLSP-E310\nname: \"E310 fixture requirement\"\nstatus: draft\nreqDomain: software\nreqClass: system\nderivedFrom: [REQ-FX-001]\n---\n\nFixture requirement missing breakdownAdr.\n",
    )
    .unwrap();

    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    let res = lsp.code_action(&path, 0, 9);
    let actions = res.as_array().expect("codeAction array");
    // Exactly one accepted ADR (ADR-FX-001) exists in the fixture model.
    let matching: Vec<&serde_json::Value> = actions
        .iter()
        .filter(|a| a.get("title").and_then(|t| t.as_str()).map(|t| t.contains("ADR-FX-001")).unwrap_or(false))
        .collect();
    assert_eq!(matching.len(), 1, "exactly one action for the one accepted ADR: {actions:?}");

    let edit = matching[0].get("edit").expect("edit present");
    let changes = edit.get("changes").and_then(|c| c.as_object()).expect("changes map");
    let (_, edits) = changes.iter().next().expect("one file edited");
    let new_text = edits[0].get("newText").and_then(|t| t.as_str()).expect("newText");
    assert!(new_text.contains("breakdownAdr:") && new_text.contains("ADR-FX-001"), "inserts breakdownAdr: {new_text}");

    lsp.shutdown();
}

// ---- TC-TRS-LSP-015: W090 quick-fix ------------------------------------------

#[test]
fn w090_gets_an_accept_as_reviewed_command_action_that_clears_on_execute() {
    let model = fixture_copy();
    let path = model.join("Requirements/REQ-FXLSP-W090.md");
    std::fs::write(
        &path,
        "---\ntype: Requirement\nid: REQ-FXLSP-W090\nname: \"W090 fixture requirement\"\nstatus: draft\nreqDomain: software\nreqClass: system\nderivedFrom: [REQ-FX-001]\nbreakdownAdr: Decisions::ADR-FX-001\ntraceBaselines:\n  REQ-FX-001: \"blake3:0000000000000000000000000000000000000000000000000000000000000000\"\n---\n\nFixture requirement with a deliberately stale traceBaselines entry.\n",
    )
    .unwrap();

    let mut lsp = Lsp::start(&model);
    lsp.initialize();

    let before = lsp.code_action(&path, 0, 12);
    let actions = before.as_array().expect("codeAction array");
    let accept = actions
        .iter()
        .find(|a| a.get("title").and_then(|t| t.as_str()) == Some("Accept as reviewed"))
        .unwrap_or_else(|| panic!("Accept as reviewed action present: {actions:?}"));
    let command = accept.get("command").expect("command present");
    assert_eq!(command.get("command").and_then(|c| c.as_str()), Some("syscribe.suspectAccept"));
    let arguments = command.get("arguments").cloned().unwrap_or(serde_json::json!([]));

    lsp.execute_command("syscribe.suspectAccept", arguments);

    let after = lsp.code_action(&path, 0, 12);
    let actions_after = after.as_array().expect("codeAction array");
    assert!(
        !actions_after.iter().any(|a| a.get("title").and_then(|t| t.as_str()) == Some("Accept as reviewed")),
        "action cleared after executing the command: {actions_after:?}"
    );

    let content = std::fs::read_to_string(&path).unwrap();
    assert!(!content.contains("0000000000000000000000000000000000000000000000000000000000000000"), "the stale hash was overwritten on disk");

    lsp.shutdown();
}

// ---- TC-TRS-LSP-016: no quick-fix for anything else --------------------------

#[test]
fn clean_requirement_gets_no_code_actions() {
    let model = fixture_copy();
    let path = model.join("Requirements/REQ-FX-003.md");

    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    let res = lsp.code_action(&path, 0, 8);
    let actions = res.as_array().expect("codeAction array, not an error");
    assert!(actions.is_empty(), "no actions for a clean requirement: {actions:?}");

    lsp.shutdown();
}
