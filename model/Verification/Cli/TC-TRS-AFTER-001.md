---
type: TestCase
id: TC-TRS-AFTER-001
name: "after: takes a variable or expression, read when the timer is armed, and every consumer follows"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/after_expr.rs
verifies:
  - REQ-TRS-AFTER-001
tags:
  - state-machine
  - execution
---

```gherkin
Feature: timer durations as expressions (GH #511)

  Scenario: a variable deadline fires at its value
    Given a machine with after naming a variable of 2 s
    When 1.9 s and then 0.2 s elapse
    Then the machine leaves its state only after the second step

  Scenario: the value is read at arming, not afterwards
    Given a deadline assigned by an earlier event, and one changed while the timer runs
    Then the assigned value counts and the running timer is not re-timed

  Scenario: static checks
    Given a dimensionless variable, an undeclared name and a bad literal
    When the model is validated
    Then E935, W933 and E882 are reported respectively

  Scenario: mutation, coverage and export follow
    Given a machine with an expression timer
    Then mutate seeds scale-after mutants that the timing scenario kills, coverage shows after <expression>, and the SCXML export carries delayexpr

  Scenario: the ActionDef form
    Given an AcceptAction whose after names a parameter
    When the action runs with a 3 s input
    Then it completes after 3 s, and a negative duration is a run error
```
