---
id: TC-TRS-PLANITEM-007
type: TestCase
testLevel: L3
status: draft
name: "Verify PlanningItem's blockedBy: resolves permissively (any element kind), rejects a dangling target or a cycle, and flags status/field staleness correctly in both directions."
verifies:
  - REQ-TRS-PLANITEM-007
---

```gherkin
Feature: PlanningItem blockedBy
  Scenario: a resolving blockedBy naming another PlanningItem validates cleanly
    Given a PlanningItem, status: blocked, with blockedBy: naming a real PlanningItem
    When the model is validated
    Then no blockedBy-related error is raised

  Scenario: a resolving blockedBy naming a non-PlanningItem validates cleanly
    Given a PlanningItem, status: blocked, with blockedBy: naming a real Requirement
    When the model is validated
    Then no blockedBy-related error is raised

  Scenario: a dangling blockedBy is rejected
    Given a PlanningItem whose blockedBy: names no existing element
    When the model is validated
    Then an unresolved-blockedBy error is raised

  Scenario: a 2-node blockedBy cycle is detected gracefully
    Given two PlanningItems each naming the other as blockedBy:
    When the model is validated
    Then a cycle error is raised, with no crash

  Scenario: a non-empty blockedBy while not status: blocked is a warning
    Given a PlanningItem with a resolving blockedBy: but status: in_progress
    When the model is validated
    Then a stale-blockedBy warning is raised, and no error

  Scenario: status: blocked with no blockedBy raises nothing
    Given a PlanningItem with status: blocked and no blockedBy: entry
    When the model is validated
    Then no blockedBy-related finding is raised at all
```
