---
id: TC-TRS-DERIVED-001
type: TestCase
testLevel: L3
status: active
name: "Verify: a derived parameter is computed from the Configuration's effective values and used wherever a bound value is."
verifies:
  - REQ-TRS-DERIVED-001
sourceFile: repo:crates/syscribe/tests/derived_calcs.rs
testFunctions:
  - a_derived_parameter_is_computed_exported_and_constrained
  - a_derived_value_is_range_and_integer_checked_and_a_cycle_is_an_error
---

```gherkin
Feature: A derived parameter is computed from the Configuration's effective values and used wherever a bound value is (TC-TRS-DERIVED-001)

  Scenario: The behaviour of REQ-TRS-DERIVED-001 holds
    Given a model built to exercise it
    When the command runs
    Then the result is as the requirement states
```
