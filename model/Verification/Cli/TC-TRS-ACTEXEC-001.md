---
type: TestCase
id: TC-TRS-ACTEXEC-001
name: "action flows lower to an IR, run with token semantics, and have scenarios, coverage and mutation"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/act_run.rs
verifies:
  - REQ-TRS-ACTEXEC-001
tags:
  - action-flow
---

```gherkin
Feature: executable action flows

  Scenario: lowering
    Then compile prints a hashed IR with nodes, edges, slots, flows and resolved callees, and refuses unresolved or recursive calls

  Scenario: token semantics
    Then decisions, forks, joins, merges, loops, ifs, performs, accepts, sends and terminate behave as specified

  Scenario: scenarios and evidence
    Then the action vocabulary gives verdicts, E886 for anything else, and recorded evidence

  Scenario: coverage and mutation
    Then nodes, edges, decision branches, loops and guard conditions are measured and mutants are scored

  Scenario: state machines
    Then an executable ActionDef named by an entry action or effect runs on the machine's variables
```
