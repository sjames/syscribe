---
id: TC-V13-DRIFT-001
type: TestCase
name: "tc_drift"
status: active
testLevel: L3
verifies:
  - REQ-V13-001
sourceFile: mock://ok/remote2.rs
testFunctions:
  - function: "m::remote_renamed"
    scenario: "case"
---

```gherkin
Feature: tc_drift
  Scenario: case
    Given a remote source
    Then it is handled
```
