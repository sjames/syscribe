---
id: TC-TRS-BT-038
type: TestCase
testLevel: L3
status: active
name: "Verify a model using none of the extensions is unchanged"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-038.sh
verifies:
  - REQ-TRS-BT-038
---

```gherkin
Feature: Additive migration
  Scenario: no new findings
    Given model/ and every examples/ model
    When validate is run
    Then no new finding appears

  Scenario: existing suites
    Given the existing variability suites
    When they are run
    Then they pass unchanged
```
