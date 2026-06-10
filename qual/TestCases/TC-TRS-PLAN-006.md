---
id: TC-TRS-PLAN-006
type: TestCase
testLevel: L3
status: draft
title: "Verify the --plan lens on matrix, verification-depth and audit: row restriction, scoped verdict, composition, unknown-id exit."
verifies:
  - REQ-TRS-PLAN-006
---

```gherkin
Feature: --plan lens
  Scenario: row restriction and composition
    Given a model with a TestPlan and an out-of-scope requirement
    When matrix --plan and verification-depth --plan are invoked
    Then rows are restricted to the plan's in-scope requirements
    And --plan composes with --config
    And an unknown plan id exits 1

  Scenario: audit --plan scopes the verdict (GH #40)
    Given a model with an out-of-scope element that has a real error
    When audit is invoked whole-model
    Then it FAILs
    When audit --plan is invoked on a plan that excludes the broken element
    Then it PASSes (the out-of-scope finding is not counted; full-model validation avoids escaping-ref artifacts)
    When audit --plan is invoked on a plan that includes the broken element
    Then it FAILs (the in-scope finding counts)
```
