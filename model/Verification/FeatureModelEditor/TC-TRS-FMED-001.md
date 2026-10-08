---
id: TC-TRS-FMED-001
type: TestCase
testLevel: L3
status: active
name: "Verify the feature diagram: derived from a feature subtree, in FODA notation, rendered by every writer, served at /features, collapsible and searchable."
verifies:
  - REQ-TRS-FMED-001
sourceFile: repo:crates/syscribe-model/tests/vis_feature_model.rs
testFunctions:
  - a_feature_model_diagram_is_derived_and_validates_clean
  - a_feature_subject_draws_its_subtree_and_a_wrong_subject_is_w418
  - the_sprotty_model_marks_features_and_flags_constraints_as_overlay
  - every_writer_renders_the_feature_diagram
  - the_whole_model_diagram_has_no_diagram_element_behind_it
tags:
  - feature-model
  - variability
---

Run with `cargo test -p syscribe-model --test vis_feature_model` and `cargo test -p syscribe-model --lib vis::derive::feature`. The page and endpoints are checked by `cargo test -p syscribe-server --test feature_model_page`, and collapse, search and reveal by `frontend/test/feature-core.test.mjs` (`npm test` in `crates/syscribe-server/frontend/`).

```gherkin
Feature: feature diagram (TC-TRS-FMED-001)

  Scenario: derived
    When a FeatureModel diagram has a feature or package subject
    Then it draws every feature under it with tree edges and constraint edges, and a subject of another type is W418

  Scenario: notation
    Then mandatory, group kind and child count reach the sprotty model and constraints are overlay edges

  Scenario: writers
    Then Mermaid, PlantUML and SVG render it, the SVG with marks and group wedges

  Scenario: the page
    Then /features and the diagram and export endpoints are served, and collapse, search and reveal behave
```
