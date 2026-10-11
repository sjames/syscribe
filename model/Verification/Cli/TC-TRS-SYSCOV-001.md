---
type: TestCase
id: TC-TRS-SYSCOV-001
name: "Composite scenarios report coverage per part and in aggregate, with per-part gates"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/syscomp_cov.rs
verifies:
  - REQ-TRS-SYSCOV-001
tags:
  - coverage
---

```gherkin
Feature: Composite coverage

  Scenario: per part and aggregate
    Then each part with a machine has its own report, labelled composite.part, and the JSON carries the aggregate of states, transitions and guard conditions

  Scenario: scenario subsets
    Then running one scenario covers less than running both, per part

  Scenario: gates
    Then a part below --min-states fails the gate with exit 2 and is named
```
