---
id: TC-TRS-VIS-029
type: TestCase
testLevel: L3
status: active
name: "Verify IBD grouping and containment: nesting by depth, deployment containment, safety and optional marks, domain frames, W420 and an unchanged default."
verifies:
  - REQ-TRS-VIS-029
sourceFile: repo:crates/syscribe-model/tests/vis_grouping.rs
testFunctions:
  - firmware_is_drawn_inside_the_mcu_inside_the_board
  - at_depth_one_the_firmware_goes_into_the_board_that_holds_the_mcu
  - safety_level_outlines_and_applies_when_badges_optional
  - group_by_domain_frames_the_parts
  - unknown_group_by_is_w420
  - edges_cross_nesting_through_ports
  - without_the_new_fields_the_part_tree_is_as_before
  - a_definition_that_contains_itself_is_opened_once_and_depth_is_capped
  - each_package_goes_into_the_nearest_host_and_packages_pair_with_hosts
  - members_whose_ids_collide_are_all_drawn
  - child_part_files_are_nested_at_depth_two
  - asil_badges_keep_the_decomposition_and_drop_a_repeated_prefix
  - group_by_outside_a_derived_ibd_is_w420
  - plantuml_and_dot_draw_marks_and_containment
  - a_projected_variant_badges_nothing_optional
tags:
  - diagrams
---

Hosted in `crates/syscribe-model/tests/vis_grouping.rs`
(`cargo test -p syscribe-model --test vis_grouping`), on an inverter with a board holding an MCU,
firmware allocated to the MCU, a SIL 2 stage and an optional arc-fault part. The grid-tie
acceptance case is `crates/syscribe/tests/ibd_grouping_gti.rs` (`GtiIbd` with and without
`--config CONF-GTI-IN-M2-001`).

```gherkin
Feature: IBD grouping and containment (TC-TRS-VIS-029)

  Scenario: containment
    Then the firmware is drawn inside the MCU inside the board at depth 2, and inside the board at depth 1

  Scenario: marks and frames
    Then the SIL 2 stage is outlined, the arc-fault part is badged optional and groupBy domain frames the hardware

  Scenario: defaults
    Then an unknown groupBy is W420 and a model without these fields draws the same part tree as before

  Scenario: robustness
    Then a self-containing definition is opened once, each board's firmware goes into its own MCU, colliding ids are all drawn and child Part files nest

  Scenario: writers and variants
    Then PlantUML and DOT draw the marks and containment, and a projected variant badges nothing optional
```
