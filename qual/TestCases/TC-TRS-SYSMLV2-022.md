---
id: TC-TRS-SYSMLV2-022
type: TestCase
testLevel: L3
status: active
name: "Verify an ingested SysMLv2 view def/view becomes a native ViewDef/View carrying expose, viewpoint and rendering, participating in W500/W502 exactly like a hand-authored view."
verifies:
  - REQ-TRS-SYSMLV2-020
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_views.rs
testFunctions:
  - a_view_def_and_view_usage_become_real_elements_with_expose_viewpoint_rendering
  - a_view_nested_inside_a_part_def_becomes_a_real_element
  - a_view_nested_inside_a_part_usage_is_its_own_element
  - w500_and_w502_fire_on_synthesized_output_exactly_as_on_hand_authored
  - a_clean_view_raises_no_w500_or_w502
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_views.rs`; run with `cargo test -p syscribe-model --test sysmlv2_views`.

```gherkin
Feature: an ingested SysMLv2 view def/view becomes a native ViewDef/View carrying expose (TC-TRS-SYSMLV2-022)

  Scenario: a view def and a view usage become native elements
    Given a view def and a view usage with expose, satisfy-viewpoint and render clauses
    When the tool ingests the model
    Then a ViewDef and a View exist and the view carries plain-string expose entries, viewpoint and rendering

  Scenario: views nested in a part def are reachable and in a part usage are not
    Given views declared in a part def body and a part usage body
    When the tool ingests the model
    Then the part def one becomes a real element and the part usage one stays invisible

  Scenario: W500 and W502 apply to synthesized views
    Given an ingested view with a dangling viewpoint or expose target, and a clean view
    When the tool validates the model
    Then W500 and W502 fire exactly as for a hand-authored view and a clean view raises neither
```
