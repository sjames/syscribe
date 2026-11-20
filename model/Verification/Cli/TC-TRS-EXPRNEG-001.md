---
id: TC-TRS-EXPRNEG-001
type: TestCase
testLevel: L3
status: active
name: "Verify the minus of a negative Celsius or Fahrenheit literal belongs to the number and the parser is accurate."
verifies:
  - REQ-TRS-EXPRNEG-001
sourceFile: repo:crates/syscribe-model/tests/expr_accuracy.rs
testFunctions:
  - a_celsius_reading_is_its_kelvin_value_whatever_its_sign
  - a_fahrenheit_reading_is_its_kelvin_value_whatever_its_sign
  - a_negative_reading_equals_the_freezing_point_moved_down_by_a_difference
  - the_two_scales_agree_where_they_meet
  - readings_below_zero_order_correctly
  - differences_between_readings_are_differences_not_readings
  - a_temperature_below_absolute_zero_is_an_error_not_a_negative_kelvin
  - negating_a_temperature_expression_stays_an_error
  - the_sign_may_be_separated_from_the_number
  - recorded_values_read_negative_readings
  - other_negative_literals_keep_their_tree
  - a_minus_before_a_unit_literal_means_what_it_always_meant
  - precedence_associativity_and_signs
  - comparisons_and_logic_with_signs
  - thousands_of_generated_expressions_agree_with_a_separate_evaluator
  - a_sweep_of_readings_in_both_scales_matches_the_conversion_formulas
---

```gherkin
Feature: TC-TRS-EXPRNEG-001

  Scenario: negative readings
    Given the model or expression the tests build
    When the command runs
    Then the result is as the requirement states
```
