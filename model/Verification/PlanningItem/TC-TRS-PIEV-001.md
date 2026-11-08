---
type: TestCase
id: TC-TRS-PIEV-001
name: "Active TestCases without planning evidence are warned about and reported"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/planning_testcase_evidence.rs
verifies:
  - REQ-TRS-PIEV-001
tags:
  - planning
---

```gherkin
Feature: TestCases recorded against a plan

  Scenario: policy off
    Then no W319 is raised without a [planning] table or with require_testcase_evidence = false

  Scenario: policy on
    Then W319 is raised on the active TestCase no PlanningItem's evidence names, and not on one named by id or qualified name, nor on a draft one

  Scenario: malformed planning table
    Then W322 is raised for an unknown key, a non-boolean value or a non-table, and the entry is ignored

  Scenario: coverage report
    Then coverage --json lists the same TestCases as testCasesWithoutPlanningEvidence

  Scenario: baseline-relative gating
    Then validate --baseline-file accepts the existing gap and fails on a new active TestCase
```
