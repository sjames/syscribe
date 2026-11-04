---
type: TestCase
id: TC-TRS-MXGAP-001
name: "Each gap cell carries its action and ids"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/matrix_gaps.rs
verifies:
  - REQ-TRS-MXGAP-001
tags:
  - cli
---

```gherkin
Feature: actionable matrix gaps

  Scenario: the four actions
    Then a requirement with no test says create-test with a suggested id, a draft-only one activate-test, a failing one fix-failing-test and one whose test does not apply widen-test

  Scenario: flat view and MCP
    Then the flat view lists gaps with a null configuration and the MCP tool filters them to one configuration
```
