---
type: TestCase
id: TC-TRS-STYPE-002
name: "Statically detectable type and unit errors in executable action flows are E935"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/stype_actions.rs
verifies:
  - REQ-TRS-STYPE-002
tags:
  - expressions
---

```gherkin
Feature: Static typing of action flows

  Scenario: guards, conditions and assignments
    Then a succession guard that is not boolean, an if or loop condition that is a number, and an assignment of the wrong kind or dimension are E935

  Scenario: arguments and nested blocks
    Then perform and send arguments and expressions inside nested blocks are checked

  Scenario: lenient where unknown
    Then parameters without a default, pins and accept bindings never cause E935, and a well-typed action validates clean

  Scenario: compile and simulate agree
    Then compile refuses an action with E935 the way validate reports it
```
