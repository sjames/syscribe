---
id: TC-TRS-RESCOORD-001
type: TestCase
testLevel: L3
status: active
name: "Verify result records carry condition and unit and are keyed by them."
verifies:
  - REQ-TRS-RESCOORD-001
sourceFile: repo:crates/syscribe/tests/results_records_coordinates.rs
testFunctions:
  - results_at_different_conditions_are_kept_side_by_side
  - a_failure_at_one_condition_is_not_hidden_by_a_pass_at_another
  - results_on_different_units_do_not_replace_each_other
  - unit_of_must_resolve_to_a_part_definition_and_is_stored_by_qualified_name
  - a_malformed_condition_rejects_the_whole_file
  - the_flags_fill_in_what_a_record_leaves_out_and_never_override_it
  - bad_flags_are_refused_and_write_nothing
  - the_flags_are_for_records_like_formats_only
  - records_without_coordinates_behave_as_before
---

```gherkin
Feature: Condition and unit on result records (TC-TRS-RESCOORD-001)

  Scenario: Results at different conditions are kept side by side
    Given records at the cold, nominal and hot ambient points
    When a later ingest records hot again
    Then only the hot record is superseded

  Scenario: Results on different units do not replace each other
    Given records for units SN-1 and SN-2
    When SN-2 is ingested again
    Then SN-1 keeps its record

  Scenario: A unit of a part definition that does not exist is refused
    Given a record whose unit names an unknown part definition
    When it is ingested
    Then the file is rejected and nothing is written
```
