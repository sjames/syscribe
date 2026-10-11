---
type: TestCase
id: TC-TRS-SMSCEN-001
name: "Gherkin scenarios on a TestCase run against its state machine and are recorded as evidence"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/sm_scen.rs
verifies:
  - REQ-TRS-SMSCEN-001
tags:
  - state-machine
---

```gherkin
Feature: executable scenarios

  Scenario: verdicts
    Then passing scenarios exit 0, a false Then is a failure with expected and actual, and a run error or unusable step is an error

  Scenario: vocabulary
    Then a step outside the closed vocabulary is E886 and an error verdict, never skipped

  Scenario: outlines
    Then each Examples row is its own scenario named with its row number

  Scenario: selection
    Then a machine runs all its scenario TestCases, --scenario filters, and an unknown scenario is a usage error

  Scenario: evidence
    Then --record writes verdicts and traces, keeps other scenarios' verdicts, and records failures as failures
```
