---
id: TC-TRS-PLANCOV-002
type: TestCase
testLevel: L3
status: active
name: "Verify a TestPlan is judged against its declared conditions, units and rigs."
verifies:
  - REQ-TRS-PLANCOV-002
sourceFile: repo:crates/syscribe/tests/plan_coverage_verdict.rs
testFunctions:
  - a_plan_that_never_ran_one_of_its_conditions_is_incomplete_not_pass
  - a_failure_at_any_point_fails_the_plan
  - without_a_declaration_the_verdict_is_what_it_was
  - min_units_needs_distinct_passing_units
  - every_rig_needs_every_point
  - testplan_detail_lists_the_gaps_and_json_carries_them
  - an_approved_plan_with_gaps_raises_w617_and_a_complete_one_does_not
  - the_plan_list_shows_the_same_verdict
---

```gherkin
Feature: Plan verdict against required coverage (TC-TRS-PLANCOV-002)

  Scenario: A condition that never ran leaves the plan incomplete
    Given a plan requiring cold, nominal and hot with passes at cold and nominal only
    When the plan verdict is read
    Then it is incomplete and the hot gap is listed

  Scenario: Distinct units are needed
    Given a plan requiring minUnits 2 and passes on one unit
    When a second unit passes
    Then the plan passes

  Scenario: An approved plan with a gap raises W617
    Given an approved plan with an uncovered point
    When the model is validated
    Then W617 lists the missing cells
