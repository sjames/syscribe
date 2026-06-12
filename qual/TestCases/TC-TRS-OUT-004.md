---
id: TC-TRS-OUT-004
type: TestCase
testLevel: L3
status: draft
name: "Verify that the tool exits non-zero when any Error-severity finding is present."
verifies:
  - REQ-TRS-OUT-004
---

Verify that the tool exits non-zero when any Error-severity finding is present.

```gherkin
Feature: Non-zero exit on errors

  Scenario: Model with one error produces non-zero exit code
    Given a model that triggers at least one E-code finding
    When the tool is invoked
    Then the exit code is non-zero (verified via $? in shell)

  Scenario: Model with errors and warnings still exits non-zero
    Given a model that triggers both E-code and W-code findings
    When the tool is invoked
    Then the exit code is non-zero
```
