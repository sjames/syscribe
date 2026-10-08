---
id: TC-TRS-FMED-003
type: TestCase
testLevel: L3
status: active
name: "Verify the configurator: propagation, conflict explanation, product count, stored configurations and a saved completion that the analysis accepts."
verifies:
  - REQ-TRS-FMED-003
sourceFile: repo:crates/syscribe-server/tests/feature_model_page.rs
testFunctions:
  - configure_propagates_a_choice_and_counts_the_products_left
  - configure_explains_a_conflict_with_the_choices_and_constraints_at_fault
  - stored_configurations_are_listed_for_loading
  - a_saved_completion_is_a_valid_configuration_the_analysis_accepts
tags:
  - feature-model
  - variability
---

Run with `cargo test -p syscribe-server --test feature_model_page`. The engine is checked by
`cargo test -p syscribe-model --lib analysis_json` (propagation, minimal conflict, count, unknown
features, stored configurations) and the click cycle, overlay, count text and conflict wording by
`frontend/test/feature-core.test.mjs`.

```gherkin
Feature: configurator (TC-TRS-FMED-003)

  Scenario: propagation
    When a feature is selected
    Then what it requires is forced on, the rest of an alternative group forced off, and the product count falls

  Scenario: conflict
    When the choices cannot be satisfied
    Then only the clashing choices and their constraints are reported

  Scenario: saving
    When the completed product is written as a Configuration
    Then the analysis calls it valid and it is listed for loading
```
