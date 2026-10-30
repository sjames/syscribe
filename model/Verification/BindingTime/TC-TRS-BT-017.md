---
id: TC-TRS-BT-017
type: TestCase
testLevel: L3
status: draft
name: "Verify parameterConstraints are evaluated three-valued on a partial binding"
sourceFile: repo:qual/tests/tc/TC-TRS-BT-017.sh
verifies:
  - REQ-TRS-BT-017
---

```gherkin
Feature: Partial-binding constraints
  Scenario: violated by the bound subset
    Given a + b <= 10 with a bound to 12 and b open
    When the model is checked
    Then the constraint is reported violated

  Scenario: undetermined
    Given the same constraint with a bound to 2 and b open
    When the model is checked
    Then nothing is reported
```
