---
id: TC-TRS-PLANITEM-006
type: TestCase
testLevel: L3
status: draft
name: "Verify the leaf-evidence rule fires only for a leaf PlanningItem at status: done with no non-waived, resolving evidence, and never for a non-leaf."
verifies:
  - REQ-TRS-PLANITEM-006
---

```gherkin
Feature: PlanningItem leaf-evidence rule
  Scenario: a leaf item marked done with resolving evidence validates cleanly
    Given a leaf PlanningItem at status: done with a resolving evidence: entry
    When the model is validated
    Then no leaf-evidence error is raised for that item

  Scenario: a leaf item marked done with no evidence at all is rejected
    Given a leaf PlanningItem at status: done with no evidence: field
    When the model is validated
    Then a leaf-evidence error is raised

  Scenario: a leaf item marked done with only rationale-waived evidence is still rejected
    Given a leaf PlanningItem at status: done whose evidence: entries are all rationale:-waived
    When the model is validated
    Then a leaf-evidence error is raised

  Scenario: a leaf item not marked done raises nothing regardless of evidence
    Given leaf PlanningItems at status: todo, in_progress, and blocked, each with no evidence
    When the model is validated
    Then no leaf-evidence error is raised for any of them

  Scenario: a non-leaf item marked done raises nothing regardless of its own evidence
    Given a non-leaf PlanningItem (has a child) at status: done with no evidence of its own
    When the model is validated
    Then no leaf-evidence error is raised for that item
```
