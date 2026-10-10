---
type: TestCase
id: TC-TRS-PHOLDFIELD-001
name: "whole-value placeholders in typed fields parse, project per configuration and are range-checked"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/placeholders_fields.rs
verifies:
  - REQ-TRS-PHOLDFIELD-001
tags:
  - cli
---

```gherkin
Feature: placeholders in typed fields

  Scenario: base model
    Then a numeric or enumerated field holding a whole-value placeholder does not fail parsing or validation

  Scenario: projection
    Then each configuration sees its own substituted, typed value

  Scenario: invalid values
    Then an out-of-domain value is E247 for the configuration and the field stays unset

  Scenario: other fields
    Then a placeholder in a field outside the list keeps its ordinary behaviour
```
