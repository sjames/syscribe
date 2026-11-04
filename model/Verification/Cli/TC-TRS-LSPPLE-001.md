---
type: TestCase
id: TC-TRS-LSPPLE-001
name: "Completion offers features, parameter paths, enumerations and stages with documentation"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/lsp_ple_completion.rs
verifies:
  - REQ-TRS-LSPPLE-001
tags:
  - lsp
---

```gherkin
Feature: product-line completion

  Scenario: features and parameters
    Then feature keys offer qualified names and ids, binding keys offer parameter paths with type, unit, range and window, and a binding value offers the enumeration

  Scenario: stages and expressions
    Then stage names come from the vocabulary and appliesWhen, requires, excludes and constraint expressions complete in context

  Scenario: nothing else changes
    Then the field's own name offers nothing and the earlier cross-reference and status completion is unchanged
```
