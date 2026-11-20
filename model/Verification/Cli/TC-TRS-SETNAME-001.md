---
id: TC-TRS-SETNAME-001
type: TestCase
testLevel: L3
status: active
name: "Verify set name= and evidence.add path=repo: behave as the requirement states."
verifies:
  - REQ-TRS-SETNAME-001
sourceFile: repo:crates/syscribe/tests/set_name_evidence.rs
testFunctions:
  - name_is_changed_in_place_on_an_id_identified_element
  - renaming_to_the_same_name_is_a_no_op_and_dry_run_writes_nothing
  - a_name_that_is_an_identity_segment_is_a_rename_and_is_refused
  - an_empty_or_multi_line_name_is_refused
  - the_old_name_quoted_in_prose_is_pointed_out
  - an_unknown_scalar_field_still_lists_what_is_supported_including_name
  - evidence_path_accepts_a_repository_relative_path
  - evidence_path_keeps_the_other_forms_and_names_what_it_looked_for
---

```gherkin
Feature: TC-TRS-SETNAME-001

  Scenario: name and repo: evidence
    Given the model or expression the tests build
    When the command runs
    Then the result is as the requirement states
```
