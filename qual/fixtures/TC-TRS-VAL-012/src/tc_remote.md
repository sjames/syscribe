---
id: TC-V12-REMOTE-001
type: TestCase
name: "tc_remote"
status: active
testLevel: L3
verifies:
  - REQ-V12-001
sourceFile: https://example.com/x/tests.rs
testFunctions:
  - function: "m::tests::present_case"
    scenario: "case"
---

```gherkin
Feature: tc_remote
  Scenario: case
    Given a source file
    Then it resolves
```
