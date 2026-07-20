---
type: TestCase
id: TC-TRS-LSP-013
name: "codeLens shows display-only reference counts above an element"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/lsp_codelens.rs
verifies:
  - REQ-TRS-LSP-013
tags:
  - lsp
  - codelens
---

```gherkin
Feature: codeLens

  Scenario: a verified Requirement gets a lens with a display-only command
    Given a fixture model where a TestCase verifies a Requirement
    When textDocument/codeLens is requested for the Requirement's file
    Then the response includes a CodeLens whose command title mentions the verifiedBy count
    And that command's identifier is empty (no click action)
```
