//! Integration tests for the `syscribe lsp` handshake and capability advertisement.
//! Realises TC-TRS-LSP-001.

mod common;
use common::*;

// ---- TC-TRS-LSP-001: initialize handshake & capability advertisement -------

#[test]
fn initialize_advertises_the_full_v1_v2_v3_capability_set() {
    let model = fixture_copy();
    let mut lsp = Lsp::start(&model);
    let res = lsp.initialize();
    let caps = res.get("capabilities").expect("capabilities present");

    // v1 (ADR-SYS-LSP-001)
    assert!(caps.get("textDocumentSync").is_some(), "textDocumentSync advertised");
    assert!(caps.get("hoverProvider").is_some(), "hoverProvider advertised");
    assert!(caps.get("definitionProvider").is_some(), "definitionProvider advertised");
    assert!(caps.get("referencesProvider").is_some(), "referencesProvider advertised");
    assert!(caps.get("workspaceSymbolProvider").is_some(), "workspaceSymbolProvider advertised");

    // v2 (ADR-SYS-LSP-002)
    assert!(caps.get("completionProvider").is_some(), "completionProvider advertised");
    assert!(caps.get("renameProvider").is_some(), "renameProvider advertised");

    // v3 (ADR-SYS-LSP-003)
    assert!(caps.get("codeLensProvider").is_some(), "codeLensProvider advertised");
    assert!(caps.get("codeActionProvider").is_some(), "codeActionProvider advertised");
    assert!(caps.get("executeCommandProvider").is_some(), "executeCommandProvider advertised");
    let commands = caps
        .get("executeCommandProvider")
        .and_then(|e| e.get("commands"))
        .and_then(|c| c.as_array())
        .expect("commands array");
    assert!(commands.iter().any(|c| c.as_str() == Some("syscribe.suspectAccept")));

    lsp.shutdown();
}

#[test]
fn clean_shutdown_exits_zero() {
    let model = fixture_copy();
    let mut lsp = Lsp::start(&model);
    lsp.initialize();
    let code = lsp.shutdown();
    assert_eq!(code, Some(0), "process exits 0 after shutdown/exit");
}
