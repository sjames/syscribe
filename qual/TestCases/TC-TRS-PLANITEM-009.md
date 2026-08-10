---
id: TC-TRS-PLANITEM-009
type: TestCase
testLevel: L3
status: draft
name: "Verify `template PlanningItem` prints a ready-to-fill skeleton, and PlanningItem is listed among the Known types shown for an unrecognized template argument."
verifies:
  - REQ-TRS-PLANITEM-009
---

```gherkin
Feature: template PlanningItem
  Scenario: template PlanningItem prints a ready-to-fill skeleton
    Given the syscribe CLI
    When "template PlanningItem" is run
    Then the output declares type: PlanningItem, a PI-* id, and a status field
    And the command exits 0

  Scenario: template is case-insensitive on the type name
    Given the syscribe CLI
    When "template planningitem" is run
    Then the output declares type: PlanningItem

  Scenario: PlanningItem is listed as a known native type
    Given the syscribe CLI
    When "template NotAType" is run
    Then the Known types listing's Native elements line includes PlanningItem
    And the command exits non-zero
```
