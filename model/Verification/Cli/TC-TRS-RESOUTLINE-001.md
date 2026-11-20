---
id: TC-TRS-RESOUTLINE-001
type: TestCase
testLevel: L3
status: active
name: "Verify the rows of a Scenario Outline recorded through records give the outline its verdict."
verifies:
  - REQ-TRS-RESOUTLINE-001
sourceFile: repo:crates/syscribe/tests/results_records_outline.rs
testFunctions:
  - a_failing_row_fails_the_outline_whatever_else_was_recorded
  - a_later_pass_at_one_condition_does_not_hide_a_fail_at_another
  - the_outline_passes_only_when_every_row_has_passed
  - a_row_that_did_not_conclude_is_not_a_pass
  - a_waived_row_shows_as_a_waiver_never_as_a_pass
  - the_outlines_own_record_still_speaks_when_no_row_was_recorded
  - an_outline_record_of_fail_survives_passing_rows
  - a_row_the_examples_table_does_not_have_does_not_count_towards_the_outline
  - rows_are_numbered_across_several_examples_tables
  - plain_scenarios_and_outlines_are_judged_together
  - the_verdict_is_judged_per_rig_when_a_plan_names_several
  - testplan_and_audit_see_the_rows_too
---

```gherkin
Feature: Outline rows through records (TC-TRS-RESOUTLINE-001)

  Scenario: Rows recorded through records give the outline its verdict
    Given a TestCase with a Scenario Outline of three example rows
    When rows are recorded as "<title> [row N]"
    Then a failing row fails the outline, it passes only when every row has passed, and trace, testplan and audit agree
```
