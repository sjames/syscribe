---
id: TC-V13-FAIL-001
type: TestCase
name: "tc_fail"
status: active
testLevel: L3
verifies:
  - REQ-V13-001
sourceFile: mock://fail/remote.rs
testFunctions:
  - function: "m::remote_present"
    scenario: "case"
---

```gherkin
Feature: tc_fail
  Scenario: case
    Given a remote source
    Then it is handled
```
