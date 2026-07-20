//! Integration tests for the `syscribe lsp` handshake and capability advertisement.
//! Realises TC-TRS-LSP-001.

mod common;
use common::*;

// ---- TC-TRS-LSP-001: initialize handshake & capability advertisement -------

#[test]
fn initialize_advertises_only_v1_capabilities() {
    let model = fixture_copy();
    let mut lsp = Lsp::start(&model);
    let res = lsp.initialize();
    let caps = res.get("capabilities").expect("capabilities present");

    assert!(caps.get("textDocumentSync").is_some(), "textDocumentSync advertised");
    assert!(caps.get("hoverProvider").is_some(), "hoverProvider advertised");
    assert!(caps.get("definitionProvider").is_some(), "definitionProvider advertised");
    assert!(caps.get("referencesProvider").is_some(), "referencesProvider advertised");
    assert!(caps.get("workspaceSymbolProvider").is_some(), "workspaceSymbolProvider advertised");

    // v2/v3 capabilities must not be advertised yet (ADR-SYS-LSP-001 phased scope).
    assert!(caps.get("completionProvider").is_none(), "completionProvider not yet implemented");
    assert!(caps.get("renameProvider").is_none(), "renameProvider not yet implemented");
    assert!(caps.get("codeLensProvider").is_none(), "codeLensProvider not yet implemented");
    assert!(caps.get("codeActionProvider").is_none(), "codeActionProvider not yet implemented");

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
