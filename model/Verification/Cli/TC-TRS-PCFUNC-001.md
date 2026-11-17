---
id: TC-TRS-PCFUNC-001
type: TestCase
testLevel: L3
status: active
name: "Verify: a parameterConstraints expression that calls a function is evaluated, and one with no truth value is an error."
verifies:
  - REQ-TRS-PCFUNC-001
sourceFile: repo:crates/syscribe/tests/derived_calcs.rs
testFunctions:
  - a_function_call_in_a_constraint_is_evaluated_and_fails
  - a_true_function_constraint_passes_and_an_unevaluable_one_is_an_error
---

```gherkin
Feature: A parameterConstraints expression that calls a function is evaluated, and one with no truth value is an error (TC-TRS-PCFUNC-001)

  Scenario: The behaviour of REQ-TRS-PCFUNC-001 holds
    Given a model built to exercise it
    When the command runs
    Then the result is as the requirement states
```
