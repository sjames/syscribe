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

  Scenario: --config projects the verdict onto a variant (GH #35)
    Given a model with a SIL-4 requirement gated by appliesWhen to an unselected feature
    When audit is invoked whole-model
    Then it FAILs (the gated requirement trips W306) and exits 2
    When audit --config <the config that excludes the feature> is invoked
    Then it PASSes (the gated requirement is projected out) and exits 0

  Scenario: --all-configs audits every configuration
    When audit --all-configs is invoked on the variant model
    Then a per-configuration verdict is printed and the overall exit reflects the worst case

  Scenario: audit --config agrees with validate --config (GH #36)
    Given a variant model with a TestCase that verifies a requirement projected out of the config
    When validate --config <the config> is invoked
    Then it is clean (exit 0)
    When audit --config <the config> is invoked
    Then danglingTestCases.count is 0 and the verdict is PASS (no phantom Error finding)

  Scenario: a parent requirement is excluded from the orphan sets (GH #37)
    Given a model with a parent requirement whose two leaves are each satisfied and verified
    When audit --json is invoked
    Then orphans.unsatisfiedRequirements.ids does not contain the parent id
    And orphans.unverifiedRequirements.ids does not contain the parent id
    And validate emits no W300 or W002 finding for the parent
    And the audit and validate views agree (parent satisfied/verified transitively)
```
