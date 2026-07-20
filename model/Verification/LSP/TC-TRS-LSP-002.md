---
type: TestCase
id: TC-TRS-LSP-002
name: "Validation findings are published as scoped, correctly-mapped LSP diagnostics"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/lsp_diagnostics.rs
verifies:
  - REQ-TRS-LSP-002
tags:
  - lsp
  - diagnostics
---

Verifies that validator findings are published via `textDocument/publishDiagnostics`, with
correct code/severity/range mapping, scoped to the file the finding is actually located in
(not necessarily the edited file), and republished on the documented triggers.

```gherkin
Feature: Diagnostics from the model validator

  Scenario: opening a file with a validator error publishes a diagnostic
    Given a fixture model containing an element with a known validator error (e.g. E025 title
      field usage)
    When the file is opened via textDocument/didOpen
    Then a publishDiagnostics notification is sent for that file's URI
    And the diagnostic's code is "E025" and its severity is Error
    And the diagnostic's range spans the element's YAML frontmatter block

    Examples:
      | code | severity |
      | E025 | Error    |
      | W042 | Warning  |

  Scenario: an external edit introduces a finding located in a different file
    Given a fixture model where file A verifies an element in file B, and file A is open
    When file B is modified on disk so that A's reference now dangles, and a
      didChangeWatchedFiles notification is sent for file B
    Then a publishDiagnostics notification is sent for file A's URI, not file B's
    And file A's diagnostics set includes the new dangling-reference finding

  Scenario: fixing a finding clears its diagnostic
    Given an open file with a published diagnostic
    When the file is rewritten on disk to fix the underlying issue and a didSave notification
      is sent for it
    Then a subsequent publishDiagnostics notification for that file has an empty diagnostics
      list for the fixed finding

  Scenario: diagnostics do not change on didChange alone
    Given an open file with no diagnostics
    When a didChange notification is sent with buffer content that introduces a validator
      error, but the file is not saved
    Then no publishDiagnostics notification reflecting that error is sent
```
