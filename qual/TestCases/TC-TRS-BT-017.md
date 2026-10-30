---
id: TC-TRS-BT-017
type: TestCase
testLevel: L3
status: active
name: "Verify parameterConstraints are evaluated three-valued on a partial binding"
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

  Scenario: a default a later stage can still change is not a point
    Given a parameter with a default whose window extends beyond the Configuration's stage
    When a constraint over it is evaluated
    Then the parameter spans its range and the constraint is undetermined, not violated

  Scenario: a default is final once its window has closed
    Given a parameter with a default whose window closed before the Configuration's stage
    When a constraint over it is evaluated
    Then the default is its value

  Scenario: a narrowing along the chain tightens an open parameter
    Given a base Configuration that narrows a parameter's range
    When a constraint over it is evaluated
    Then the narrowed range decides the constraint
```
