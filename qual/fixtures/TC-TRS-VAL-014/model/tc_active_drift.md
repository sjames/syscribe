---
id: TC-V14-AD-001
type: TestCase
name: "tc_active_drift"
status: active
testLevel: L3
verifies:
  - REQ-V14-001
sourceFile: model:tests.rs
testFunctions:
  - function: "m::renamed"
    scenario: "case"
---

```gherkin
Feature: tc_active_drift
  Scenario: case
    Given x
    Then y
```
