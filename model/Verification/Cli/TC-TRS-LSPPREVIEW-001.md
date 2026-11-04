---
type: TestCase
id: TC-TRS-LSPPREVIEW-001
name: "Preview offers a report and an edit without writing; apply commits only clean changes and undoes"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/lsp_preview.rs
verifies:
  - REQ-TRS-LSPPREVIEW-001
tags:
  - lsp
---

```gherkin
Feature: preview before write

  Scenario: preview
    Then a bind or scaffold is reported with its diff and findings, offers an edit that yields exactly what the CLI writes, and nothing is written

  Scenario: apply and undo
    Then a clean change is committed and its undo restores the file or deletes the created one

  Scenario: rejection
    Then a change that adds an error, or a malformed request, is refused and the model directory is unchanged
```
