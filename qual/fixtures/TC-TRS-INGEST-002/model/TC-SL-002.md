---
id: TC-SL-002
type: TestCase
testLevel: L1
status: active
name: "Automated"
verifies: [REQ-SL-002]
testFunctions:
  - function: "crate::tests::it_works"
    scenario: "A scenario"
---

```gherkin
Feature: automated
  Scenario: A scenario
    Given x
    When y
    Then z
```
