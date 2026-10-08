---
id: TC-TRS-VIS-025
type: TestCase
testLevel: L3
status: active
name: "Verify a hand-listed Sequence diagram is placed, labelled and drawn like a derived one, keeps an author's pins, and that the demo model's MissionExecutionSeq exports."
verifies:
  - REQ-TRS-VIS-025
sourceFile: repo:crates/syscribe-model/tests/vis_derive_sequence.rs
testFunctions:
  - a_hand_listed_sequence_diagram_is_fully_pinned_with_waypoints_on_every_message
  - messages_are_rows_in_declaration_order_stem_to_stem_and_self_messages_loop
  - unlabelled_messages_are_labelled_by_their_ref_and_labels_are_kept
  - fragments_box_their_messages_and_the_outer_one_is_wider
  - several_activations_on_one_lifeline_share_its_rows_in_order
  - a_diagram_with_any_pin_keeps_its_authors_geometry
  - it_renders_without_elk_and_never_fails_for_a_manifest_sequence_diagram
  - the_demo_models_hand_listed_sequence_diagram_exports
tags:
  - diagram
  - visualisation
  - sequence
---

Hosted in the `manifest_sequence` module of `crates/syscribe-model/tests/vis_derive_sequence.rs`
(run with `cargo test -p syscribe-model --test vis_derive_sequence`). The layout-option rule
(`fixed` only when fully pinned) is asserted by `vis::sprotty`'s unit test of the root options.

```gherkin
Feature: placing a hand-listed sequence diagram (TC-TRS-VIS-025)

  Scenario: no geometry in the manifest
    Given a Sequence diagram of lifelines, an actor, activations, fragments and edges with no layout
    When its IR is built
    Then every shape is pinned and every message has waypoints, columns in declaration order

  Scenario: messages
    Then rows go down in declaration order, run stem to stem, a self message loops and unlabelled messages take their ref's last segment

  Scenario: fragments and activations
    Then fragments enclose their messages with the outer wider, and two activations on one lifeline share its rows without overlap

  Scenario: an author's pins win
    Given the diagram pins one shape
    Then nothing is placed

  Scenario: drawing
    Then it renders without ELK, and the demo model's MissionExecutionSeq builds, is fully pinned and exports
```
