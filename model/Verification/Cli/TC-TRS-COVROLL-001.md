---
type: TestCase
id: TC-TRS-COVROLL-001
name: "matrix --rollup lists rows, verdicts and the per-class footer"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/coverage_rollup.rs
verifies:
  - REQ-TRS-COVROLL-001
tags:
  - cli
---

```gherkin
Feature: matrix roll-up

  Scenario: rows and footer
    Then each requirement has own/below/verdict and the footer counts verdicts per class

  Scenario: filters, policy and json
    Then --status filters rows, a rollup rule changes the parent verdict and --json carries rows and byClass
```
