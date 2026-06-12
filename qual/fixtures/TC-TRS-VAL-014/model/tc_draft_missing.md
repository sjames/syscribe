---
id: TC-V14-DM-001
type: TestCase
name: "tc_draft_missing"
status: draft
testLevel: L3
verifies:
  - REQ-V14-001
sourceFile: model:gone.rs
testFunctions:
  - function: "m::present_case"
    scenario: "case"
---

```gherkin
Feature: tc_draft_missing
  Scenario: case
    Given x
    Then y
```
