---
id: TC-TRS-PLANITEM-008
type: TestCase
testLevel: L3
status: draft
name: "Verify PlanningItem's assignedTo: enforces a Unix-style username format unconditionally, and checks roster membership only when [users] is configured, without double-reporting a single defect."
verifies:
  - REQ-TRS-PLANITEM-008
---

```gherkin
Feature: PlanningItem assignedTo
  Scenario: a well-formed username in a configured roster validates cleanly
    Given a PlanningItem with assignedTo: naming a username present in [users]
    When the model is validated
    Then no assignedTo-related error is raised

  Scenario: a well-formed username not in the roster is rejected
    Given a PlanningItem with assignedTo: naming a username absent from a non-empty [users]
    When the model is validated
    Then an undeclared-user error is raised

  Scenario: a malformed username is rejected regardless of roster configuration
    Given a PlanningItem with assignedTo: set to a value that is not a valid username
    When the model is validated
    Then a malformed-username error is raised, and no undeclared-user error is also raised

  Scenario: assignedTo is unchecked for roster membership when [users] is not configured
    Given a PlanningItem with a well-formed assignedTo: and no [users] table at all
    When the model is validated
    Then no undeclared-user error is raised

  Scenario: a malformed [users] key is flagged and excluded from the roster
    Given a .syscribe.toml [users] table with one malformed key alongside well-formed ones
    When the model is validated
    Then a malformed-roster-key warning is raised, and the well-formed entries still validate correctly
```
