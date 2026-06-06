---
id: TC-V12-MODEL-001
type: TestCase
title: "tc_model"
status: draft
testLevel: L3
verifies:
  - REQ-V12-001
sourceFile: model:tests.rs
testFunctions:
  - function: "m::tests::present_case"
    scenario: "case"
---

```gherkin
Feature: tc_model
  Scenario: case
    Given a source file
    Then it resolves
```
