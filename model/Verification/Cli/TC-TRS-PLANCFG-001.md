---
id: TC-TRS-PLANCFG-001
type: TestCase
testLevel: L3
status: active
name: "Verify a TestPlan is judged on the results of the Configurations it names."
verifies:
  - REQ-TRS-PLANCFG-001
sourceFile: repo:crates/syscribe/tests/plan_config_verdict.rs
testFunctions:
  - a_failure_recorded_for_another_configuration_does_not_touch_the_plan
  - a_failure_recorded_for_the_plans_own_configuration_fails_it
  - a_pass_recorded_for_another_configuration_does_not_complete_the_plan
  - results_ingested_without_a_configuration_still_count_for_the_plan
  - a_plan_without_configurations_keeps_the_conservative_verdict
  - several_configurations_are_each_judged_and_shown
  - a_plan_without_required_coverage_is_scoped_the_same_way
  - a_member_is_judged_only_in_the_configurations_it_is_part_of
  - validate_gates_an_approved_plan_on_its_own_configurations_only
  - audit_counts_the_plan_by_the_same_verdict
---

```gherkin
Feature: Plan verdict per Configuration (TC-TRS-PLANCFG-001)

  Scenario: Another variant's failure does not touch the plan
    Given a plan for CONF-HTR-001 that passes at every required point
    When a failing record is ingested for CONF-HTR-002
    Then the plan still passes and lists no gap

  Scenario: Each listed Configuration is judged
    Given a plan for two Configurations with passes for the first only
    When the plan verdict is read
    Then it is incomplete and the gaps name the second Configuration

  Scenario: A plan without configurations is conservative
    Given a plan that lists no Configuration
    When a failing record is ingested for any Configuration
    Then the plan fails
```
