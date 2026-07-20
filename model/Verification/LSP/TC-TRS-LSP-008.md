---
type: TestCase
id: TC-TRS-LSP-008
name: "Completion offers type-filtered id/qname candidates for cross-reference fields"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/lsp_completion.rs
verifies:
  - REQ-TRS-LSP-008
tags:
  - lsp
  - completion
---

```gherkin
Feature: Field-aware completion

  Scenario: completion inside a verifies list offers only Requirement candidates
    Given a fixture model with several Requirements and a TestCase with a `verifies:` list
    When textDocument/completion is requested at the cursor position on a new `verifies:` entry
    Then every returned candidate's label resolves to a Requirement
    And no candidate resolves to a non-Requirement element

  Scenario: completion inside a breakdownAdr field offers only ADR candidates
    Given a fixture model with an ADR and a Requirement's `breakdownAdr:` field
    When textDocument/completion is requested at the cursor position on that field's value
    Then every returned candidate's label resolves to an ADR
```
