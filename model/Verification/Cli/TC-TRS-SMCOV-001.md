---
type: TestCase
id: TC-TRS-SMCOV-001
name: "simulate --coverage measures states, transitions, pairs, guard conditions with MC/DC, timers and requirements"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/sm_cov.rs
verifies:
  - REQ-TRS-SMCOV-001
tags:
  - state-machine
---

```gherkin
Feature: scenario coverage

  Scenario: structure
    Then states, transitions and transition pairs are counted with the uncovered ones listed

  Scenario: guard conditions
    Then each atomic condition is seen true and false and an independent-effect demonstration is reported

  Scenario: timers
    Then an after trigger is covered only when it both fired and was cancelled

  Scenario: requirements and basis
    Then requirements count only when every scenario of a verifying TestCase passes, and the basis is stated

  Scenario: gates
    Then the minimum thresholds decide the exit code
```
