---
id: TC-V12-MISS-001
type: TestCase
title: "tc_missing"
status: draft
testLevel: L3
verifies:
  - REQ-V12-001
sourceFile: model:gone.rs
testFunctions:
  - function: "m::tests::present_case"
    scenario: "case"
---

```gherkin
Feature: tc_missing
  Scenario: case
    Given a source file
    Then it resolves
```
