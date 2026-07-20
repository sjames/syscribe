---
type: TestCase
id: TC-TRS-LSP-009
name: "Completion offers enum-value candidates for type and status fields"
status: draft
testLevel: L2
sourceFile: repo:crates/syscribe/tests/lsp_completion.rs
verifies:
  - REQ-TRS-LSP-009
tags:
  - lsp
  - completion
---

```gherkin
Feature: Enum-field completion

  Scenario: completion on a status field offers that type's status domain
    Given a fixture Requirement's `status:` field
    When textDocument/completion is requested at the cursor position on that field's value
    Then the candidate labels are exactly the Requirement status enum (draft, review, approved,
      implemented, verified)

  Scenario: completion on a type field offers every known element type
    Given any element's `type:` field
    When textDocument/completion is requested at the cursor position on that field's value
    Then the candidate labels include "Requirement", "TestCase", and "ADR"
```
