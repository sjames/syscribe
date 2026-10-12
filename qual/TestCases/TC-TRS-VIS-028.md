---
id: TC-TRS-VIS-028
type: TestCase
testLevel: L3
status: active
name: "Verify flow kinds and port sides: resolution order, styling in every writer, the [diagram] table, port-side rules and W419."
verifies:
  - REQ-TRS-VIS-028
sourceFile: repo:crates/syscribe-model/tests/vis_flow.rs
testFunctions:
  - flow_comes_from_the_entry_else_the_port_else_its_port_def
  - a_model_without_flow_kinds_draws_as_before
  - flow_kinds_style_every_writer
  - plantuml_carries_the_kind_as_a_stereotype
  - the_diagram_table_restyles_a_kind
  - port_side_rule_places_ports_by_flow_and_a_port_side_wins
  - config_parses_styles_and_rules
  - unknown_kinds_sides_and_table_entries_are_w419
  - plantuml_thickness_is_an_integer_and_signal_keeps_the_plain_arrow
  - mermaid_styles_only_edges_whose_kind_changes_their_look_at_their_own_index
  - dot_draws_data_dotted_and_command_dashed
  - a_binding_carries_no_flow_kind
  - a_port_def_inherits_its_supertypes_flow_kind
  - a_manifest_diagram_takes_the_flow_kind_of_its_ports
  - w419_covers_every_place_a_value_is_read_or_ignored
tags:
  - diagrams
---

Hosted in `crates/syscribe-model/tests/vis_flow.rs`
(`cargo test -p syscribe-model --test vis_flow`), on an inverter IBD with a power, a command and
a signal connection.

```gherkin
Feature: flow kinds (TC-TRS-VIS-028)

  Scenario: kinds and styles
    Then each edge takes the entry's kind, else its port's, else the PortDef's, and every writer draws it

  Scenario: project table
    When the project restyles power and sets port sides by kind
    Then the new stroke is drawn, ports move to their sides and a port's own side wins

  Scenario: bad values and unchanged models
    Then an unknown kind, side or table entry is W419 and ignored, and a model with no kinds draws as before
```
