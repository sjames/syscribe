---
id: TC-OUT9-PASS-001
type: TestCase
testLevel: L3
status: approved
name: "Test for the passing requirement"
verifies:
  - REQ-OUT9-PASS-001
testFunctions:
  - function: "widget::tests::pass_ok"
    scenario: "It passes"
---

```gherkin
Feature: TC-OUT9-PASS-001
  Scenario: nominal
    Given the system
    Then REQ-OUT9-PASS-001 is satisfied
```
