---
id: TC-V14-AM-001
type: TestCase
name: "tc_active_missing"
status: active
testLevel: L3
verifies:
  - REQ-V14-001
sourceFile: model:gone.rs
testFunctions:
  - function: "m::present_case"
    scenario: "case"
---

```gherkin
Feature: tc_active_missing
  Scenario: case
    Given x
    Then y
```
