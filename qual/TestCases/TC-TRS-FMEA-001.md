---
id: TC-TRS-FMEA-001
type: TestCase
testLevel: L3
status: draft
name: "Verify that FMEASheet and FMEAEntry validation rules E911–E914, W902–W904 are enforced."
verifies:
  - REQ-TRS-FMEA-001
---

Verify that the tool detects and reports every validation error and warning defined for the `FMEASheet` and `FMEAEntry` element types.

```gherkin
Feature: FMEASheet and FMEAEntry validation rule enforcement

  Scenario Outline: Each FMEA validation code is produced by its trigger condition
    Given a model fixture that satisfies the trigger condition for <code>
    When the tool validates the model
    Then at least one <code> finding is emitted

    Examples:
      | code  | trigger condition                                                                         |
      | E911  | FMEASheet element is missing one or more of id, title, status                             |
      | E912  | FMEASheet id is present but does not match the FMEA-* pattern                             |
      | E913  | FMEAEntry id is present but does not match the FM-* pattern                               |
      | E914  | FMEAEntry fmeaSeverity, occurrence, or detection value is outside range 1–10              |
      | W902  | FMEASheet has no entries field                                                             |
      | W903  | FMEAEntry RPN is greater than 100 but has no recommendedAction field                      |
      | W904  | FMEAEntry ref field does not resolve to a known element                                    |
```
