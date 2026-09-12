---
id: TC-TRS-PLANITEM-010
type: TestCase
testLevel: L3
status: draft
name: "Verify W310 fires for a done PlanningItem's under-verified achieves requirement, respecting the leaf/parent W002/W305 bar, and never for todo/in_progress/blocked."
verifies:
  - REQ-TRS-PLANITEM-010
---

```gherkin
Feature: PlanningItem-scoped completion check (W310)
  Scenario: a done PlanningItem achieves a leaf requirement with no active TestCase
    Given a done PlanningItem that achieves a leaf Requirement with only a draft TestCase
    When the model is validated
    Then W310 is raised naming the PlanningItem and the Requirement

  Scenario: a done PlanningItem achieves a leaf requirement with an active TestCase
    Given a done PlanningItem that achieves a leaf Requirement with an active TestCase
    When the model is validated
    Then no W310 is raised for that PlanningItem

  Scenario: a done PlanningItem achieves a parent requirement with only leaf-level active coverage
    Given a done PlanningItem that achieves a parent Requirement whose only active TestCase is testLevel L1
    When the model is validated
    Then W310 is raised, matching W305's own integration-level bar for a parent Requirement

  Scenario: a done PlanningItem achieves a parent requirement with active integration-level coverage
    Given a done PlanningItem that achieves a parent Requirement with an active testLevel L3 TestCase
    When the model is validated
    Then no W310 is raised for that PlanningItem

  Scenario: a non-done PlanningItem is never checked
    Given PlanningItems at status todo, in_progress, and blocked, each achieving an under-verified leaf Requirement
    When the model is validated
    Then no W310 is raised for any of them

  Scenario: a dangling or wrong-kind achieves target is not re-flagged by W310
    Given a done PlanningItem whose achieves entry is dangling or resolves to a non-Requirement element
    When the model is validated
    Then E714 or E715 is raised as usual, and W310 is not also raised for that entry
```
