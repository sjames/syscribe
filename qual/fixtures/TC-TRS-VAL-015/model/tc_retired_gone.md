---
id: TC-V15-RET-001
type: TestCase
name: "tc_retired_gone"
status: retired
testLevel: L5
verifies:
  - REQ-V15-001
sourceFile: model:removed.rs
testFunctions:
  - function: "m::future_case"
    scenario: "case"
---

```gherkin
Feature: tc_retired_gone
  Scenario: case
    Given planned hardware
    Then it will be tested
```
