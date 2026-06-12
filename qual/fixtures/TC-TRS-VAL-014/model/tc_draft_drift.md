---
id: TC-V14-DD-001
type: TestCase
name: "tc_draft_drift"
status: draft
testLevel: L3
verifies:
  - REQ-V14-001
sourceFile: model:tests.rs
testFunctions:
  - function: "m::renamed"
    scenario: "case"
---

```gherkin
Feature: tc_draft_drift
  Scenario: case
    Given x
    Then y
```
