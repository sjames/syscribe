---
id: TC-TRS-PLANITEM-001
type: TestCase
testLevel: L3
status: draft
name: "Verify PlanningItem's PI-* id pattern and its status/itemType vocabularies are each independently enforced."
verifies:
  - REQ-TRS-PLANITEM-001
---

```gherkin
Feature: PlanningItem id/status/itemType schema
  Scenario: a valid PlanningItem validates cleanly
    Given a PlanningItem with a valid PI-* id, name, status, and itemType
    When the model is validated
    Then no PlanningItem schema errors are raised

  Scenario: a malformed id is rejected
    Given a PlanningItem whose id does not match the PI-* pattern
    When the model is validated
    Then an id-pattern error is raised

  Scenario: a missing required field is rejected
    Given a PlanningItem with no status
    When the model is validated
    Then a required-field error is raised

  Scenario: an out-of-vocabulary status is rejected
    Given a PlanningItem with status: wontfix
    When the model is validated
    Then a status-vocabulary error is raised

  Scenario: an out-of-vocabulary itemType is rejected
    Given a PlanningItem with itemType: epic
    When the model is validated
    Then an itemType-vocabulary error is raised
```
