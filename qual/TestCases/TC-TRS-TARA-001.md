---
id: TC-TRS-TARA-001
type: TestCase
testLevel: L3
status: draft
name: "Verify that TARASheet validation rules E940–E941, W905 are enforced."
verifies:
  - REQ-TRS-TARA-001
---

Verify that the tool detects and reports every validation error and warning defined for the `TARASheet` element type.

```gherkin
Feature: TARASheet validation rule enforcement

  Scenario Outline: Each TARA validation code is produced by its trigger condition
    Given a model fixture that satisfies the trigger condition for <code>
    When the tool validates the model
    Then at least one <code> finding is emitted

    Examples:
      | code  | trigger condition                                                                         |
      | E940  | TARASheet element is missing one or more of id, title, status                             |
      | E941  | TARASheet id is present but does not match the TARA-* pattern                             |
      | W905  | TARASheet has no rows in any of damageTable, threatTable, goalTable, or controlTable      |
```
