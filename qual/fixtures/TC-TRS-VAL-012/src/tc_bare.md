---
id: TC-V12-BARE-001
type: TestCase
title: "tc_bare"
status: draft
testLevel: L3
verifies:
  - REQ-V12-001
sourceFile: tests.rs
testFunctions:
  - function: "m::tests::present_case"
    scenario: "case"
---

```gherkin
Feature: tc_bare
  Scenario: case
    Given a source file
    Then it resolves
```
