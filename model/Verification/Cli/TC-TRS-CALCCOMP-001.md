---
id: TC-TRS-CALCCOMP-001
type: TestCase
testLevel: L3
status: active
name: "Verify: a calculation composes: other calculations are read by name or called with arguments, and a used calculation raises no W007."
verifies:
  - REQ-TRS-CALCCOMP-001
sourceFile: repo:crates/syscribe/tests/derived_calcs.rs
testFunctions:
  - a_calculation_names_and_calls_other_calculations
  - a_calculation_cycle_and_a_bad_argument_are_errors
---

```gherkin
Feature: A calculation composes: other calculations are read by name or called with arguments, and a used calculation raises no W007 (TC-TRS-CALCCOMP-001)

  Scenario: The behaviour of REQ-TRS-CALCCOMP-001 holds
    Given a model built to exercise it
    When the command runs
    Then the result is as the requirement states
```
