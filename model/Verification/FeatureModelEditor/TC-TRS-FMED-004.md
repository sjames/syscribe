---
id: TC-TRS-FMED-004
type: TestCase
testLevel: L3
status: active
name: "Verify feature-model editing: every operation and its undo, refusals, the validity delta, confirmation of a worsening edit, reference rewriting and the undo history."
verifies:
  - REQ-TRS-FMED-004
sourceFile: repo:crates/syscribe-model/tests/feature_edit.rs
testFunctions:
  - add_creates_a_feature_file_with_a_fresh_id_and_undo_deletes_it
  - add_with_a_group_kind_and_membership_and_by_parent_id
  - a_new_root_goes_beside_the_existing_roots_and_ids_never_collide
  - add_refuses_a_duplicate_a_bad_name_an_unknown_parent_and_a_bad_group
  - set_group_changes_the_key_and_undo_restores_the_file
  - set_mandatory_toggles_the_flag_and_drops_the_legacy_shorthand
  - constraints_are_added_by_stable_id_refused_when_repeated_and_removed
  - remove_a_leaf_cleans_constraints_and_configuration_keys_and_undo_restores_everything
  - remove_a_feature_with_children_needs_the_subtree_flag
  - rename_changes_the_qualified_name_rewrites_references_and_the_label_and_undo_renames_back
  - renaming_a_feature_that_is_a_file_with_a_directory_of_children_moves_both
  - renaming_a_group_in_the_index_layout_moves_its_directory
  - move_reparents_a_feature_with_its_subtree_and_undo_moves_it_back_refusing_a_cycle
  - a_root_can_be_made_a_child_and_a_child_a_root
  - a_sheet_entry_takes_group_and_membership_edits_and_refuses_the_rest
  - the_validity_delta_names_what_an_edit_made_dead_and_what_undoing_it_fixed
  - the_validity_delta_reports_a_model_made_void_and_an_invalidated_configuration
  - an_edit_that_changes_nothing_for_validity_does_not_worsen_it
  - a_sheet_entry_takes_group_and_membership_edits
  - renaming_a_sheet_entry_renames_its_subtree_keeps_ids_and_rewrites_every_reference
  - a_sheet_entry_moves_within_its_sheet_with_its_subtree_and_constraints_that_name_it_by_path
  - a_sheet_entry_cannot_leave_its_sheet_or_collide_and_says_why
  - sheet_constraints_are_added_inline_and_removed_from_the_cross_tree_list
  - removing_sheet_entries_takes_the_subtree_the_cross_tree_constraint_and_the_choices
  - parameters_are_added_replaced_and_removed_with_their_bindings_in_either_layout
  - removing_a_feature_also_removes_the_bindings_of_its_parameters
tags:
  - feature-model
  - variability
---

Run with `cargo test -p syscribe-model --test feature_edit`. The endpoint, preview, confirmation and
undo round trip are checked by `cargo test -p syscribe-server --test feature_model_edit`; the
map-key rewriting by `cargo test -p syscribe-model --lib key_rewrite`; the bound-parameter rule through the endpoint by `a_bound_parameter_cannot_be_removed_until_its_binding_is`; the MCP tool by `cargo test -p syscribe --test mcp_feature_edit`; the undo history, delta
wording and drop target by `frontend/test/feature-core.test.mjs` (`npm test` in
`crates/syscribe-server/frontend/`).

```gherkin
Feature: feature model editing (TC-TRS-FMED-004)

  Scenario: operations and undo
    Then each operation edits the right files and its undo restores the model byte for byte

  Scenario: refusals
    Then a duplicate, a bad name, a cycle, a subtree without the flag or a sheet entry says why and changes nothing

  Scenario: validity
    When an edit makes a feature dead, forced on, the model void or a configuration invalid
    Then the delta says so and the edit is held until confirmed

  Scenario: references
    When a feature is renamed or moved
    Then every reference is rewritten, including the keys of a configuration
```
