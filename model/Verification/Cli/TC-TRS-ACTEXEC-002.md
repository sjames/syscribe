---
type: TestCase
id: TC-TRS-ACTEXEC-002
name: "simulate, scenarios, coverage and mutate work for ActionDefs from the command line"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/act_cli.rs
verifies:
  - REQ-TRS-ACTEXEC-001
tags:
  - action-flow
---

```gherkin
Feature: action flows on the command line

  Scenario: script and scenarios
    Then simulate runs a script and scenarios with pass, fail and error verdicts and records evidence

  Scenario: coverage
    Then nodes, edges, decision branches and guard conditions are reported and the gates set the exit code

  Scenario: mutation
    Then mutate scores the scenarios and the minimum score gates the exit code
```
