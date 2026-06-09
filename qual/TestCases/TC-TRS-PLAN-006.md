---
id: TC-TRS-PLAN-006
type: TestCase
testLevel: L3
status: draft
title: "Verify the --plan lens on matrix and verification-depth: row restriction, --config composition, unknown-id exit."
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
```
