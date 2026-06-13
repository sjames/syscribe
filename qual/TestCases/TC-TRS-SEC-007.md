---
id: TC-TRS-SEC-007
type: TestCase
name: "W039 fires for CAL3 CybersecurityGoal with no I2 confirmation measure"
status: active
testLevel: L1
verifies: [REQ-TRS-SEC-007]
---

Verify W039 fires for a CAL3 CybersecurityGoal with no I2 confirmation measure, and that an I2 (or I3) CM clears the warning.

```gherkin
Feature: CAL-aware confirmation measure gate (W039)

  Scenario: CAL3 goal without an I2 confirmation measure
    Given a CAL3 CybersecurityGoal with no I2 cybersecurity_assessment ConfirmationMeasure
    When I validate the model
    Then the output contains "W039"

  Scenario: an I2 confirmation measure clears the warning
    Given a CAL3 CybersecurityGoal confirmed by an I2 (or I3) cybersecurity_assessment
    When I validate the model
    Then no W039 warning is reported
```
