---
id: TC-TRS-SEC-006
type: TestCase
name: "derivedFromCybersecurityGoal field resolves correctly; alias derivedFromSecurityGoal also works"
status: active
testLevel: L1
verifies: [REQ-TRS-SEC-006]
---

Verify the renamed `derivedFromCybersecurityGoal` field resolves, the legacy `derivedFromSecurityGoal` alias still works, and a dangling reference triggers E831.

```gherkin
Feature: derivedFromCybersecurityGoal resolution and legacy alias

  Scenario: canonical field resolves
    Given a Requirement with derivedFromCybersecurityGoal pointing to a CybersecurityGoal
    When I validate the model
    Then no E831 error is reported

  Scenario: legacy alias still resolves
    Given a Requirement using the legacy derivedFromSecurityGoal alias
    When I validate the model
    Then no E831 error is reported

  Scenario: dangling reference is rejected
    Given a Requirement with derivedFromCybersecurityGoal pointing to a missing element
    When I validate the model
    Then the output contains "E831"
```
