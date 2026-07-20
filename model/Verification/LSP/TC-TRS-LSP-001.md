---
type: TestCase
id: TC-TRS-LSP-001
name: "lsp subcommand completes the LSP initialize handshake over stdio, advertising only implemented capabilities"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/lsp_handshake.rs
verifies:
  - REQ-TRS-LSP-001
tags:
  - lsp
---

Verifies that `syscribe lsp` starts a server that speaks LSP over stdio, completes the
`initialize`/`initialized` handshake, advertises only the capabilities actually implemented in
this release, exposes no custom (non-LSP) methods, and shuts down cleanly.

```gherkin
Feature: LSP server startup and handshake

  Scenario: initialize handshake succeeds over stdio
    Given a fixture model directory
    When syscribe lsp -m <fixture> is spawned and an initialize request is sent on stdin
    Then a JSON-RPC initialize response is returned on stdout
    And the response's capabilities include textDocumentSync, hoverProvider,
      definitionProvider, referencesProvider, and workspaceSymbolProvider
    And the response's capabilities do not include completionProvider, renameProvider,
      codeLensProvider, or codeActionProvider
    And no protocol bytes are written to stderr

  Scenario: no custom JSON-RPC methods are required
    Given the server has completed the initialize/initialized handshake
    When only standard LSP requests are sent for the remainder of the session
    Then the client never needs to send a non-LSP-spec method to use any capability

  Scenario: clean shutdown
    Given the server has completed the initialize/initialized handshake
    When a shutdown request is sent followed by an exit notification
    Then the process exits with code 0

  Scenario: model root resolution matches other model-bound commands
    Given a fixture model directory with a .syscribe.toml at its root
    When syscribe lsp is spawned from a working directory inside the fixture without -m
    Then the server binds to the model root resolved by walking up to .syscribe.toml
```
