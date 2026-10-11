---
type: TestCase
id: TC-TRS-STYPE-001
name: "Statically detectable type and unit errors in executable machines are E935"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/stype.rs
verifies:
  - REQ-TRS-STYPE-001
tags:
  - expressions
---

```gherkin
Feature: Static typing of executable expressions

  Scenario: dimension and kind errors
    Then a guard comparing metres with a plain number, a guard that is not boolean, an assignment of the wrong dimension and a boolean operator over a number are E935

  Scenario: functions
    Then min over unequal dimensions, sqrt of an odd dimension and a non-integer pow exponent are E935

  Scenario: lenient where unknown
    Then payload fields and other names that are not variables never cause E935, and machines that are not executable are not checked

  Scenario: compile and simulate agree
    Then compile refuses a machine with E935 the same way validate reports it
```
