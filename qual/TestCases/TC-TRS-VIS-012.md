---
id: TC-TRS-VIS-012
type: TestCase
testLevel: L2
status: active
name: "Verify vis::style resolves one visual language — node colours by element type then kind with unresolved dashed, the spec 8.16.8 edge table row by row for all twenty-four kinds, port glyphs by direction — and serialises it camelCase with explicit nulls."
verifies:
  - REQ-TRS-VIS-012
sourceFile: repo:crates/syscribe-model/src/vis/style.rs
testFunctions:
  - node_style_prefers_element_type_then_kind_and_dashes_unresolved
  - edge_style_follows_the_spec_tables_row_by_row
  - port_style_by_direction
  - styles_serialise_camel_case
tags:
  - diagram
  - visualisation
---

Unit tests in `crates/syscribe-model/src/vis/style.rs`; run with
`cargo test -p syscribe-model vis::style`. Each test calls `node_style`, `edge_style` or
`port_style` directly on hand-built IR values and compares against the colours, arrowheads,
dash patterns, keywords and glyphs spec §8.16.8 fixes.

That the resolved style is carried on every node, port and edge of the sprotty graph is covered
by `nodes_ports_and_edges_carry_their_resolved_style` in `crates/syscribe-model/src/vis/sprotty.rs`
(`cargo test -p syscribe-model vis::sprotty`) and, end to end through the route, by
`TC-TRS-VIS-006`.

```gherkin
Feature: one visual language in Rust (TC-TRS-VIS-012)

  Scenario: node colours come from the element type, then the kind, and unresolved is dashed
    Given block nodes typed PartDef, Part, Requirement, TestCase, PortDef, Interface, ConnectionDef, ActionDef, State, UseCaseDef and Allocation
    When their styles are resolved
    Then PartDef and Part share one style with no header fill, Requirement and TestCase carry a header fill, and each other type has its own fill and stroke
    Given a Package drawn as a boundary and untyped nodes of every kind
    When their styles are resolved
    Then the Package falls back to the boundary colours and each kind has its own fill and stroke
    Given a resolved and an unresolved PartDef node
    Then only the unresolved one is dashed

  Scenario: edge styles follow the spec table row by row
    Given all twenty-four edge kinds
    When each style is resolved
    Then stroke, dash, target arrowhead, source arrowhead and keyword match the spec row for every kind
    And connection has no arrowhead, flow a filled one, binding is dashed with keyword =, inheritance a hollow triangle, composition a filled diamond at the source, aggregation a hollow diamond, and the traceability kinds an open arrow with their keyword
    And every kind shares one stroke width

  Scenario: port glyphs follow the direction
    When styles are resolved for in, out, inout and no direction
    Then the glyphs are in, out, inout and none, with an out port filled dark and the others light

  Scenario: styles serialise camelCase with explicit nulls
    When a composition, an inheritance and an untyped block style are serialised to JSON
    Then arrowSource is filledDiamond and arrowTarget none for composition, arrowTarget is hollowTriangle for inheritance
    And an absent dash, keyword and headerFill are null, and dashed is false
```
