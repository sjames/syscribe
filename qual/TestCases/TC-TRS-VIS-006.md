---
id: TC-TRS-VIS-006
type: TestCase
testLevel: L3
status: active
name: "Verify GET /api/diagrams/model serves a Diagram's IR as a nested sprotty graph — ports in blocks, blocks in the boundary, edges at the root, position/size only for pins, the pinned set, per-kind ELK layoutOptions and the resolved style — accepts both qname separators, derives a subject-only diagram, and 404s for Mermaid, non-diagram and unknown names."
verifies:
  - REQ-TRS-VIS-006
sourceFile: repo:crates/syscribe-server/tests/diagram_model.rs
testFunctions:
  - ibd_is_served_nested_with_pins_layout_options_and_root_edges
  - bdd_layout_options_and_shorthand_shapes
  - qname_accepts_both_separators
  - derived_diagram_returns_the_generated_graph_for_its_subject
  - mermaid_non_diagram_and_unknown_are_404
tags:
  - diagram
  - visualisation
  - sprotty
---

Hosted integration tests in `crates/syscribe-server/tests/diagram_model.rs`; run with
`cargo test -p syscribe-server --test diagram_model`. Each test drives the real router in-process
(`build_router` + `new_state`, exactly as `main` builds it) via `tower::ServiceExt::oneshot`
against the checked-in fixture model under `tests/fixtures/diagram_model`. The endpoint is
read-only, so the fixture is served in place.

The `PATCH`-`null` and `DELETE /api/diagrams/layout` half of the requirement is covered by
`TC-TRS-VIS-011` (`tests/layout_routes.rs`).

```gherkin
Feature: the diagram-model endpoint serves a nested sprotty graph (TC-TRS-VIS-006)

  Scenario: an IBD is nested, pinned where a human pinned it, and carries its layout options
    Given an IBD diagram whose manifest places a port under a block under a boundary, pins one block with w and h, and names one shape that does not resolve
    When GET /api/diagrams/model/Diagrams/SysIBD is called
    Then the root children are the boundary and the edge, in declaration order
    And the port is a child of the block and the block a child of the boundary, each with its label child first
    And the port carries its direction, a side and a style, the block its stereotype, isAbstract and style
    And only the pinned block has position and size, and pinned lists exactly that id
    And the unresolved shape is served with resolved false and a dashed style
    And the edge is a root child with its kind, source, target, routingPoints and style, and no node contains an edge
    And layoutOptions are layered, RIGHT, INCLUDE_CHILDREN, FIXED_SIDE with no reversed edge kinds

  Scenario: a BDD honours the shorthand and the map form and layers inheritance upward
    Given a BDD diagram with one shape in string shorthand and one in map form joined by an inheritance edge
    When GET /api/diagrams/model/Diagrams/SysBDD is called
    Then both shapes are block nodes with the part def stereotype
    And the inheritance edge carries a hollow-triangle arrowhead and no dash
    And layoutOptions are DOWN with no hierarchy handling and inheritance in syscribe.reversedEdgeKinds
    And pinned is empty

  Scenario: both qname spellings resolve
    When the diagram is requested as Diagrams::SysBDD and as Diagrams/SysBDD
    Then both return 200 and byte-identical JSON

  Scenario: a subject-only diagram is derived
    Given a Diagram with an IBD kind and a subject but no shapes
    When its model is requested
    Then the graph is the derived boundary of the subject with its ref, kind and isAbstract, and pinned is empty

  Scenario: nothing without an IR is served
    When a Mermaid-kind diagram, a PartDef and an unknown name are requested
    Then each returns 404
```
