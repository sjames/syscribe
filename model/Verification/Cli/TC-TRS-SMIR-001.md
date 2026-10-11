---
type: TestCase
id: TC-TRS-SMIR-001
name: "a StateDef lowers to a deterministic, hashed, source-annotated IR printed by compile"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/sm_ir.rs
verifies:
  - REQ-TRS-SMIR-001
tags:
  - state-machine
---

```gherkin
Feature: state-machine intermediate representation

  Scenario: the flat tables
    Then states have ids, parents, kinds and initial children, transitions keep document order, and variables get slots

  Scenario: triggers and actions
    Then event, timer, change and done triggers, assignments and behaviour references are lowered, with durations in seconds

  Scenario: source and hash
    Then every transition and state names its model element and YAML path, and the hash is stable and changes with the machine

  Scenario: lowering diagnostics
    Then an unresolved target, an ambiguous name, a non-expression guard and schema errors fail compilation with a code

  Scenario: usage
    Then compiling something that is not a state machine exits 1
```
