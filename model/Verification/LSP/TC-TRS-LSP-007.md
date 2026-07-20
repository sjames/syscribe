---
type: TestCase
id: TC-TRS-LSP-007
name: "Model state reloads fully on save and on external file changes, and survives a failed reload"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/lsp_reload.rs
verifies:
  - REQ-TRS-LSP-007
tags:
  - lsp
---

Verifies that the server's in-memory model state stays consistent with disk via full reload on
`didSave` and on `didChangeWatchedFiles`, that diagnostics for open documents are recomputed
after every reload, and that a failed reload leaves the previous state intact.

```gherkin
Feature: Full model reload

  Scenario: didSave triggers a full reload
    Given the server has loaded a fixture model
    When a file under the model root is saved via didSave with content that adds a new element
    Then a subsequent workspace/symbol query finds the new element

  Scenario: an external file change triggers a full reload
    Given the server has loaded a fixture model and registered for didChangeWatchedFiles
    When a file under the model root is modified on disk outside the editor (e.g. by git
      checkout) and a didChangeWatchedFiles notification is sent
    Then a subsequent workspace/symbol query reflects the externally-modified content

  Scenario: diagnostics recompute after reload
    Given an open document with no diagnostics
    When an external change introduces a validator error in that open document
    And a didChangeWatchedFiles notification triggers a reload
    Then a publishDiagnostics notification for that document reflects the new error

  Scenario: a failed reload preserves prior state
    Given the server has loaded a fixture model successfully
    When a reload is triggered while the model root is transiently unreadable
    Then the server's in-memory state remains the last successfully loaded state
    And a subsequent workspace/symbol query still returns results from that prior state
```
