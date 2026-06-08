---
id: TC-TRS-OUT-013
type: TestCase
testLevel: L3
status: draft
title: "Verify the safety-readiness audit dashboard, its sections, JSON output and PASS/FAIL exit codes."
verifies:
  - REQ-TRS-OUT-013
---

Verify that `syscribe audit` aggregates the status split, SIL/ASIL distribution,
per-configuration coverage %, orphans and a readiness verdict; that a ready model
audits PASS (exit 0) and a not-ready model with a high-integrity unsatisfied/draft
requirement audits FAIL naming W306 (exit 2); and that `audit --json` emits one
valid JSON document carrying `statusSplit`, `coverage` and `verdict`.

```gherkin
Feature: Safety-readiness audit dashboard

  Scenario: A ready model audits PASS and exits 0
    Given a model with approved requirements that are satisfied and covered and no W306
    When audit is invoked
    Then the exit code is 0
    And the report prints the requirement status split section
    And the report prints the per-configuration coverage % section
    And the report prints a verdict line stating PASS

  Scenario: A not-ready model audits FAIL and exits 2
    Given a model with a SIL-4 requirement that is draft or unsatisfied (tripping W306)
    When audit is invoked
    Then the exit code is 2
    And the verdict line states FAIL and names W306

  Scenario: audit --json emits one structured document
    Given the ready model
    When audit --json is invoked
    Then the output is valid JSON
    And it contains the keys statusSplit, coverage and verdict
```
