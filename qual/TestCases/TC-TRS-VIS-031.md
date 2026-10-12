---
id: TC-TRS-VIS-031
type: TestCase
testLevel: L3
status: active
name: "Verify diagram views: include selectors that fade the context, named styles, layout hints, W419/W421 and the two-configuration variant view."
verifies:
  - REQ-TRS-VIS-031
sourceFile: repo:crates/syscribe-model/tests/vis_views.rs
testFunctions:
  - a_kinds_view_fades_the_parts_and_edges_it_does_not_include
  - domains_tags_and_features_select_parts
  - a_named_style_hides_the_context_and_restyles_a_flow_kind
  - style_problems_are_w419_and_w421
  - view_problems_are_w421
  - layout_hints_set_the_flow_and_order_the_bands
  - the_variant_view_marks_what_only_one_product_has
  - writers_draw_faded_elements
  - layers_order_the_laid_out_bands
  - ignored_entries_select_nothing_away_and_malformed_values_are_w421
  - a_negated_feature_does_not_select_a_part
  - only_a_style_on_a_cycle_is_blamed_and_default_may_be_extended
  - writers_fade_frames_and_follow_the_flow
  - the_variant_view_marks_the_outermost_part_and_quotes_its_labels
tags:
  - diagrams
---

Hosted in `crates/syscribe-model/tests/vis_views.rs` (`cargo test -p syscribe-model --test
vis_views`). The grid-tie acceptance case is `crates/syscribe/tests/views_gti.rs`: `GtiPowerView`,
`GtiProtectionView`, `GtiSoftwareView`, and `GtiIbd` with `--config CONF-GTI-US-M3-001 --config
CONF-GTI-IN-M2-001`.

```gherkin
Feature: Diagram views (TC-TRS-VIS-031)

  Scenario: selectors
    Then a kinds view fades the parts and edges outside it, and domains, tags and features select their parts

  Scenario: styles and hints
    Then a hiding style leaves the context out and widens its flow kind, and flow and layers set direction and band order

  Scenario: problems
    Then a missing or cyclic extends is W419, and an unknown style, key, kind, domain, feature or hint is W421

  Scenario: variants and writers
    Then two configurations draw as one diagram marking what only one has, and every writer draws faded elements pale
```
