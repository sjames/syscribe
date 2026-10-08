---
id: TC-TRS-FMED-002
type: TestCase
testLevel: L3
status: active
name: "Verify the analysis overlay: dead, core, false-optional and void with reasons, served by the analysis endpoint and applied to the diagram."
verifies:
  - REQ-TRS-FMED-002
sourceFile: repo:crates/syscribe-server/tests/feature_model_page.rs
testFunctions:
  - the_analysis_endpoint_names_the_dead_feature_and_why
  - a_void_model_reports_the_conflict_and_corrections
  - a_model_without_features_is_empty_not_an_error
tags:
  - feature-model
  - variability
---

Run with `cargo test -p syscribe-server --test feature_model_page`. The engine function is checked by `cargo test -p syscribe-model --lib analysis_json` (dead, core, false-optional with reasons, void with diagnoses, and the mandatory-feature regression), and the overlay, banner and summary text by `frontend/test/feature-core.test.mjs`.

```gherkin
Feature: feature analysis overlay (TC-TRS-FMED-002)

  Scenario: states with reasons
    Then a dead feature names the excluding constraint, a core feature is core, and a feature forced by a core feature is core and false-optional

  Scenario: void
    Then the conflict and the corrections are reported

  Scenario: no feature model
    Then the report says so and is not an error

  Scenario: overlay
    Then states are applied to the diagram by qualified name, cleared without a report, and the banner and summary follow the report
```
