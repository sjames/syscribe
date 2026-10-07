---
id: TC-TRS-VIS-017
type: TestCase
testLevel: L2
status: active
name: "Verify Rust-owned sizes: the ELK input built for the fixture IBD matches its snapshot and the embedded engine's output equals the committed Node output exactly; label boxes, compartments, ports and leaf minimums follow the client's recipe; and the sprotty graph carries a size on every element."
verifies:
  - REQ-TRS-VIS-017
sourceFile: repo:crates/syscribe-model/tests/vis_layout.rs
testFunctions:
  - elk_under_quickjs_matches_node_for_the_ibd_fixture
tags:
  - diagram
  - visualisation
  - elk
---

Hosted integration test in `crates/syscribe-model/tests/vis_layout.rs`; run with
`cargo test -p syscribe-model --test vis_layout`. The determinism check builds the ELK input
for the derived IBD of the fixture `Sys::PowerSystem` with `vis::layout::elk_input` over the
approximate metrics, compares it to `tests/vis_snapshots/elk/power_ibd.input.json`, runs the
committed input through the embedded QuickJS engine and compares the result, after JSON
canonicalisation and with no tolerance, to `power_ibd.output.json` — produced from the same
input by `elkjs` under Node with the command quoted in the test. The sizing recipe is covered
by the unit tests of `crates/syscribe-model/src/vis/size.rs`
(`label_boxes_carry_the_margin_and_the_line_factor`,
`a_block_stacks_stereotype_banners_name_and_compartment_like_the_client_vbox`,
`ports_are_twelve_square_with_a_name_label_and_free_labels_are_text_boxes`,
`a_leaf_is_raised_to_the_minimum_and_to_what_its_ports_need`,
`edge_labels_follow_the_style_keyword_and_the_label`, `sizing_is_deterministic`), the carried
sizes in the sprotty graph by `position_only_for_pinned_nodes_and_size_on_every_element`,
`nodes_nest_by_parent_and_ports_sit_inside_their_block`,
`labels_and_compartments_map_to_their_own_types` and
`edges_stay_top_level_with_kind_ref_label_and_routing_points` in
`crates/syscribe-model/src/vis/sprotty.rs`, and the ELK option mirror by
`elk_input_mirrors_the_client_configurator` and
`reversed_kinds_are_flipped_and_pins_switch_the_modes` in
`crates/syscribe-model/src/vis/layout.rs` (`cargo test -p syscribe-model --lib vis::`).

```gherkin
Feature: one set of sizes, one set of coordinates (TC-TRS-VIS-017)

  Scenario: the ELK input is pinned
    Given the derived IBD of the fixture PowerSystem sized with the approximate metrics
    When the ELK input is built
    Then it equals the committed power_ibd.input.json byte for byte

  Scenario: QuickJS and Node agree
    Given the committed ELK input
    When it is laid out by the embedded engine
    Then the result equals the committed Node output after number canonicalisation, with no tolerance
    And the result carries real geometry: a position and size per block and sections per edge

  Scenario: sizes follow the client's recipe
    Given a block with a stereotype, a banner, a name and a compartment of two lines
    When the graph is sized
    Then the labels stack from y = 4 with a 1 px gap and the compartment follows, 8 px padded, 2 px gapped
    And a port is 12 by 12 with a name label, a leaf is at least 120 by 40 and 28n + 16 on a side with n ports
    And a pin recording both w and h is carried in place of the measurement

  Scenario: the sprotty graph carries the sizes
    When the IR is written as a sprotty graph
    Then every node, port, compartment and label carries size, and the synthetic labels carry role and size
```
