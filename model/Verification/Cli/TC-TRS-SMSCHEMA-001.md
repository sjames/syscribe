---
type: TestCase
id: TC-TRS-SMSCHEMA-001
name: "the extended state-machine schema validates, and malformed uses raise E882-E885 / W933"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/sm_schema.rs
verifies:
  - REQ-TRS-SMSCHEMA-001
tags:
  - state-machine
---

```gherkin
Feature: executable state-machine schema

  Scenario: a well-formed executable machine
    Then variables, after/when/done triggers, internal transitions, assign and initialEffect validate clean

  Scenario: triggers
    Then two trigger kinds, a bad duration, an unparseable when and an unknown done target are E882

  Scenario: variables and assignments
    Then bad or duplicate variable names or values are E883, an unparseable or undeclared-target assignment is E884

  Scenario: internal transitions
    Then an internal transition with a target is E885, needs no target for W929 and is not an outgoing edge

  Scenario: dormant for existing models
    Then a machine with opaque guards and none of the new keys raises nothing new
```
