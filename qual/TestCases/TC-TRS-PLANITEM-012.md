---
id: TC-TRS-PLANITEM-012
type: TestCase
testLevel: L3
status: draft
name: "Verify W311 fires once per overlapping pair of active PlanningItems (shared achieves or evidence.path), and never for inactive or non-overlapping pairs."
verifies:
  - REQ-TRS-PLANITEM-012
---

```gherkin
Feature: PlanningItem claim-overlap check (W311)
  Scenario: two in_progress items sharing an achieves requirement raise W311
    Given two in_progress PlanningItems that both achieve the same Requirement
    When the model is validated
    Then W311 is raised naming both PlanningItems and the shared Requirement

  Scenario: two items sharing an evidence path raise W311
    Given two PlanningItems whose evidence[] both reference the same path
    When the model is validated
    Then W311 is raised naming both PlanningItems and the shared path

  Scenario: a claimed-but-todo item still counts as active
    Given a todo PlanningItem with claimedBy set, and an in_progress PlanningItem, sharing an achieves requirement
    When the model is validated
    Then W311 is raised

  Scenario: two todo, unclaimed items sharing scope raise nothing
    Given two todo, unclaimed PlanningItems that share an achieves requirement
    When the model is validated
    Then no W311 is raised

  Scenario: two items with disjoint scope raise nothing
    Given two in_progress PlanningItems with no shared achieves requirement or evidence path
    When the model is validated
    Then no W311 is raised

  Scenario: W311 fires once per pair, not once per side
    Given exactly two overlapping in_progress PlanningItems
    When the model is validated
    Then exactly one W311 finding is present for the pair
```
