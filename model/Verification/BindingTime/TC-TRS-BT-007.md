---
id: TC-TRS-BT-007
type: TestCase
testLevel: L3
status: draft
name: "Verify an unknown stage or an inverted window is an error"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-007.sh
verifies:
  - REQ-TRS-BT-007
---

```gherkin
Feature: Window validity
  Scenario: unknown stage
    Given a parameter with bindingTime bogus
    When validate is run
    Then an error is raised on the parameter

  Scenario: inverted window
    Given bindingTime load with latestBindingTime compile
    When validate is run
    Then an error is raised
```
