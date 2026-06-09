---
id: TC-TRS-PLAN-005
type: TestCase
testLevel: L3
status: draft
title: "Verify the testplan command: list, detail --json contract, goal-closure in-scope, verdict roll-up."
verifies:
  - REQ-TRS-PLAN-005
---

```gherkin
Feature: testplan command
  Scenario: list and detail
    Given a model with a TestPlan whose member has an ingested passing result
    When testplan and testplan TP-X --json are invoked
    Then the list shows the plan, scope and pass verdict
    And the detail JSON carries schemaVersion, inScopeRequirements (goal-closure), effectiveTestCases, coverage and verdict
    And an unknown plan id exits non-zero
```
