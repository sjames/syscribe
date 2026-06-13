---
id: TC-TRS-SEC-005
type: TestCase
name: "ConfirmationMeasure.confirms accepts CybersecurityGoal and enforces E860 for wrong types"
status: active
testLevel: L1
verifies: [REQ-TRS-SEC-005]
---

Verify that `ConfirmationMeasure.confirms` accepts `CybersecurityGoal` targets, and that referencing an invalid target type triggers E860.

```gherkin
Feature: ConfirmationMeasure.confirms accepts CybersecurityGoal

  Scenario: confirms references a CybersecurityGoal
    Given a ConfirmationMeasure whose confirms references a CybersecurityGoal
    When I validate the model
    Then no E860 error is reported

  Scenario: confirms references a disallowed element type
    Given a ConfirmationMeasure whose confirms references an invalid target type
    When I validate the model
    Then the output contains "E860"
```
