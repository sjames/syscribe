---
id: TC-TRS-PLANITEM-003
type: TestCase
testLevel: L3
status: draft
name: "Verify achieves: is required on a top-level PlanningItem, rejects a dangling/wrong-type target, and never participates in W300/E312."
verifies:
  - REQ-TRS-PLANITEM-003
---

```gherkin
Feature: PlanningItem achieves:
  Scenario: a top-level item with a resolving achieves: validates cleanly
    Given a top-level PlanningItem with achieves: naming a real Requirement
    When the model is validated
    Then no achieves-related errors are raised for that item

  Scenario: a top-level item with no achieves: is rejected
    Given a top-level PlanningItem with no achieves: entries
    When the model is validated
    Then a missing-achieves error is raised

  Scenario: a dangling achieves: target is rejected
    Given a PlanningItem whose achieves: names no existing element
    When the model is validated
    Then an unresolved-achieves error is raised

  Scenario: a wrong-type achieves: target is rejected
    Given a PlanningItem whose achieves: names a non-Requirement element
    When the model is validated
    Then a wrong-type-achieves error is raised

  Scenario: achieves: never suppresses W300
    Given a leaf Requirement named only via a PlanningItem's achieves:, never satisfies:
    When the model is validated
    Then the leaf-requirement coverage warning is still raised

  Scenario: achieves: never triggers E312
    Given a parent Requirement (has derivedChildren) named only via achieves:, never satisfies:
    When the model is validated
    Then no no-parent-assignment error is raised for that requirement
```
