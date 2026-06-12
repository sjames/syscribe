---
id: TC-V13-OK-001
type: TestCase
name: "tc_ok"
status: active
testLevel: L3
verifies:
  - REQ-V13-001
sourceFile: mock://ok/remote.rs
testFunctions:
  - function: "m::remote_present"
    scenario: "case"
---

```gherkin
Feature: tc_ok
  Scenario: case
    Given a remote source
    Then it is handled
```
