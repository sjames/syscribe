---
id: TC-TRS-PLAN-001
type: TestCase
testLevel: L3
status: draft
name: "Verify the native TestPlan schema: TP-id pattern, status enum, scope vocabulary and duplicate-id."
verifies:
  - REQ-TRS-PLAN-001
---

```gherkin
Feature: TestPlan schema validation
  Scenario: a well-formed TestPlan parses clean
    Given a TestPlan with a valid TP-id, title, status and recommended scope
    When validate runs
    Then no E600, E604 or W610 is raised and the exit code is 0

  Scenario: schema violations are flagged
    Given a TestPlan with a malformed TP-id, one with a bad status, one with a non-recommended scope
    When validate runs
    Then E600, E604 and W610 are each raised

  Scenario: duplicate TestPlan id
    Given two TestPlans sharing one id
    When validate runs
    Then the generic E101 duplicate-id error is raised
```
