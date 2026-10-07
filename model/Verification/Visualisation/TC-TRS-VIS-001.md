---
id: TC-TRS-VIS-001
type: TestCase
testLevel: L2
status: active
name: "Verify the Diagram IR carries resolved types, labels, nesting, ports and pins from a manifest, and is the only input of the PlantUML writer."
verifies:
  - REQ-TRS-VIS-001
sourceFile: repo:crates/syscribe-model/src/vis/manifest.rs
testFunctions:
  - string_shorthand_and_map_form_both_parse
  - omitted_kind_defaults_to_block_and_omitted_edge_kind_to_connection
  - parent_nesting_and_ports_are_honoured
  - element_type_names_are_accepted_as_kinds
  - layout_pins_nodes_and_edges_and_flags_stale_keys
tags:
  - diagram
  - visualisation
---

Hosted unit tests in `crates/syscribe-model/src/vis/manifest.rs`; run with
`cargo test -p syscribe-model vis::manifest`. The IR's own value-type tests (JSON round trip,
nesting/pin helpers, per-kind layout hints) live beside it in `vis/ir.rs`, and the PlantUML
writer's IR-only input is pinned by `cargo test -p syscribe-model --test vis_plantuml_snapshot`
(every `pumlMode: companion` diagram of the demo model against
`crates/syscribe-model/tests/snapshots/plantuml/`).

```gherkin
Feature: one Diagram IR is the sole input of every renderer and exporter (TC-TRS-VIS-001)

  Scenario: shorthand and map-form shapes build typed nodes
    Given a BDD manifest with one shape in the string shorthand and one in the map form
    When the IR is built
    Then each shape is a Block node whose element type, stereotype, abstract flag and label come from the resolved element

  Scenario: an omitted kind defaults
    Given shapes and an edge that declare no kind
    When the IR is built
    Then the nodes are Block nodes and the edge is a connection

  Scenario: parent nesting and ports are honoured
    Given a boundary, a block with parent set to it, and a port with parent set to the block
    When the IR is built
    Then the block is a child of the boundary and the port is a Port node of the block carrying its direction

  Scenario: element-type names are accepted as kinds
    Given a shape with kind Part and an unresolved shape with kind Requirement
    When the IR is built
    Then the resolved element's real type wins, and the unresolved shape is kept with the type its kind implied

  Scenario: layout entries become pins
    Given a layout block pinning one shape and one edge
    When the IR is built
    Then the shape carries the pin, the edge carries the waypoints, and the graph is not fully pinned
```
