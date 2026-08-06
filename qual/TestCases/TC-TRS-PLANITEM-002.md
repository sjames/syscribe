---
id: TC-TRS-PLANITEM-002
type: TestCase
testLevel: L3
status: draft
name: "Verify PlanningItem's single-parent hierarchy resolves across multiple levels, rejects a dangling or wrong-type parent, and detects cycles gracefully."
verifies:
  - REQ-TRS-PLANITEM-002
---

```gherkin
Feature: PlanningItem single-parent hierarchy
  Scenario: a multi-level parent chain resolves correctly
    Given a 3-level chain of PlanningItems linked by parent:
    When the model is validated
    Then no hierarchy-related errors are raised

  Scenario: a dangling parent is rejected
    Given a PlanningItem whose parent: names no existing element
    When the model is validated
    Then an unresolved-parent error is raised

  Scenario: a wrong-type parent is rejected
    Given a PlanningItem whose parent: names a Requirement, not a PlanningItem
    When the model is validated
    Then a wrong-type-parent error is raised

  Scenario: a 2-node parent cycle is detected gracefully
    Given two PlanningItems each naming the other as parent:
    When the model is validated
    Then a cycle error is raised, with no crash

  Scenario: a 3-node parent cycle is detected gracefully
    Given three PlanningItems whose parent: chain cycles back to the first
    When the model is validated
    Then a cycle error is raised, with no crash
```
