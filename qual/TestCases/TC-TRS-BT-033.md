---
id: TC-TRS-BT-033
type: TestCase
testLevel: L3
status: active
name: "Verify retained elements are listed and unstaged projection is unchanged"
verifies:
  - REQ-TRS-BT-033
---

```gherkin
Feature: Retained list and compatibility
  Scenario: listing
    Given a staged Configuration with retained elements
    When validate --config and residual are run
    Then both list the same retained elements

  Scenario: unchanged
    Given model/ and the existing examples
    When they are projected
    Then the output is identical to before
```
