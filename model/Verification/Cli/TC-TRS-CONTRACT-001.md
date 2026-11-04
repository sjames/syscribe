---
type: TestCase
id: TC-TRS-CONTRACT-001
name: "Real --json output matches the published schemas"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/json_contract.rs
verifies:
  - REQ-TRS-CONTRACT-001
tags:
  - cli
---

```gherkin
Feature: published JSON contracts

  Scenario: real output conforms
    Then features, feature, configure, residual, matrix, diff, feature-check and params check output validates against its schema

  Scenario: the validator is not vacuous
    Then wrong documents are rejected and a compatible extra property is accepted

  Scenario: the schema command
    Then it lists, prints and refuses an unknown name with a suggestion
```
