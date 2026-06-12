---
id: TC-TRS-SAFE-002
type: TestCase
testLevel: L3
status: draft
name: Verify that SafetyGoal validation rules E805-E806, E825, E837, W801, W805, and W806 are enforced
verifies:
  - REQ-TRS-SAFE-002
---

Verify that the tool detects all SafetyGoal field-level and cross-reference validation errors and warnings.

```gherkin
Feature: SafetyGoal validation rules

  Scenario: E805 — missing required field triggers error
    Given a SafetyGoal element missing the id field
    When the tool validates the model
    Then at least one E805 finding is emitted

  Scenario: E806 — id not matching SG-* pattern triggers error
    Given a SafetyGoal with id that does not match the SG-* pattern
    When the tool validates the model
    Then at least one E806 finding is emitted

  Scenario: E825 — unresolvable hazardousEvents reference triggers error
    Given a SafetyGoal with a hazardousEvents entry that does not resolve to any element
    When the tool validates the model
    Then at least one E825 finding is emitted

  Scenario: E837 — invalid plLevel value triggers error
    Given a valid SafetyGoal with plLevel set to an invalid value
    When the tool validates the model
    Then at least one E837 finding is emitted

  Scenario: W801 — SafetyGoal with no integrity level triggers warning
    Given a valid SafetyGoal with no asilLevel, silLevel, or plLevel
    When the tool validates the model
    Then at least one W801 finding is emitted

  Scenario: W805 — SafetyGoal not referenced by any Requirement triggers warning
    Given a valid SafetyGoal with no Requirement pointing to it via derivedFromSafetyGoal
    When the tool validates the model
    Then at least one W805 finding is emitted

  Scenario: W806 — SafetyGoal with no hazardousEvents field triggers warning
    Given a valid SafetyGoal with no hazardousEvents field at all
    When the tool validates the model
    Then at least one W806 finding is emitted
```
