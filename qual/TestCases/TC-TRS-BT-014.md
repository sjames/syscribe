---
id: TC-TRS-BT-014
type: TestCase
testLevel: L3
status: active
name: "Verify ancestor values are closed and ranges only narrow"
verifies:
  - REQ-TRS-BT-014
---

```gherkin
Feature: Closed ancestor values
  Scenario: rebinding
    Given a child binding a parameter its base bound
    When validate is run
    Then an error is raised

  Scenario: narrow and widen
    Given a child that narrows a range and another that widens it
    When validate is run
    Then the narrowing is clean and the widening is an error
```
