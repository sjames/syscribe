---
id: TC-TRS-SEC-004
type: TestCase
name: "AOU.appliesTo accepts CybersecurityGoal and enforces E859 for wrong types"
status: active
testLevel: L1
verifies: [REQ-TRS-SEC-004]
---

Verify that `AssumptionOfUse.appliesTo` accepts `CybersecurityGoal` targets without error, and that referencing a non-allowed element type triggers E859.

```gherkin
Feature: AssumptionOfUse.appliesTo accepts CybersecurityGoal

  Scenario: appliesTo references a CybersecurityGoal
    Given an AssumptionOfUse whose appliesTo references a CybersecurityGoal
    When I validate the model
    Then no E859 error is reported

  Scenario: appliesTo references a disallowed element type
    Given an AssumptionOfUse whose appliesTo references a PartDef
    When I validate the model
    Then the output contains "E859"
```
