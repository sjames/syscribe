---
id: TC-V15-DRAFT-001
type: TestCase
name: "tc_draft_planned"
status: draft
testLevel: L5
verifies:
  - REQ-V15-001
sourceFile: model:not_yet.rs
testFunctions:
  - function: "m::future_case"
    scenario: "case"
---

```gherkin
Feature: tc_draft_planned
  Scenario: case
    Given planned hardware
    Then it will be tested
```
