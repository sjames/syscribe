---
id: TC-TRS-PLAN-003
type: TestCase
testLevel: L3
status: draft
name: "Verify TestPlan membership: E601, E602, E605, empty-set W612 and explicit-draft W613."
verifies:
  - REQ-TRS-PLAN-003
---

```gherkin
Feature: TestPlan membership
  Scenario: unresolvable explicit member
    Given a TestPlan whose testCases entry resolves to no TestCase
    When validate runs
    Then E601 is raised

  Scenario: invalid selection sub-fields
    Given a selection.testLevels outside L1-L5 and a selection.domains outside system/hardware/software
    When validate runs
    Then E602 and E605 are raised

  Scenario: empty and draft membership
    Given a plan with no effective members, and a plan that pins a draft TestCase
    When validate runs
    Then W612 and W613 are raised
```
