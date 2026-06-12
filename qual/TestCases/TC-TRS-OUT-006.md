---
id: TC-TRS-OUT-006
type: TestCase
testLevel: L3
status: draft
name: "Verify CI severity-gating flags and the 0/1/2 exit-code contract."
verifies:
  - REQ-TRS-OUT-006
---

Verify that the `validate` gating flags (`--deny`, `--max-warnings`, `--warnings-as-errors`) trip the gate with exit code `2`, that absent denied codes stay clean, and that `Error` findings dominate with exit code `1`.

```gherkin
Feature: CI severity gating and exit-code contract

  Scenario: Warnings without a gate exit zero
    Given a model that emits a W005 warning and no errors
    When validate is invoked with no gating flags
    Then the exit code is 0

  Scenario: Denying a present warning code trips the gate
    Given a model that emits a W005 warning
    When validate is invoked with --deny W005
    Then the exit code is 2

  Scenario: Denying an absent warning code stays clean
    Given a model that emits a W005 warning
    When validate is invoked with --deny W999
    Then the exit code is 0

  Scenario: Exceeding max-warnings trips the gate
    Given a model that emits a W005 warning
    When validate is invoked with --max-warnings 0
    Then the exit code is 2

  Scenario: Warnings-as-errors trips the gate
    Given a model that emits a W005 warning
    When validate is invoked with --warnings-as-errors
    Then the exit code is 2

  Scenario: Errors dominate gating flags
    Given a model that contains an E005 error
    When validate is invoked with --warnings-as-errors
    Then the exit code is 1
```
