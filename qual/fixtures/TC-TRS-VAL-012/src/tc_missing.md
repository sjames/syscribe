---
id: TC-V12-MISS-001
type: TestCase
name: "tc_missing"
status: active
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
