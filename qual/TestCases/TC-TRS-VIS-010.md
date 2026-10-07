---
id: TC-TRS-VIS-010
type: TestCase
testLevel: L2
status: active
name: "Verify the Mermaid and static SVG writers are pure functions of the IR: golden snapshots for the derived BDD/IBD fixture, a %% ref: per Mermaid node, sysml:ref per SVG node and sysml:source/target per edge, refusal of an unpinned graph, and the REQ-TRS-LINK-002 hyperlink wrapper from the links closure."
verifies:
  - REQ-TRS-VIS-010
  - REQ-TRS-VIS-009
sourceFile: repo:crates/syscribe-model/tests/vis_writers.rs
testFunctions:
  - mermaid_of_the_derived_bdd_matches_its_snapshot_and_annotates_every_node
  - mermaid_of_the_derived_ibd_matches_its_snapshot_and_nests_subgraphs
  - mermaid_click_lines_follow_the_links_closure
  - svg_of_the_pinned_derived_bdd_matches_its_snapshot_with_sysml_attributes
  - svg_of_the_pinned_derived_ibd_matches_its_snapshot_and_bounds_the_boundary
  - svg_refuses_a_graph_with_an_unpinned_node
  - svg_wraps_linked_nodes_in_the_req_trs_link_002_anchor_and_nothing_else
tags:
  - diagram
  - visualisation
---

Hosted integration tests in `crates/syscribe-model/tests/vis_writers.rs`; run with
`cargo test -p syscribe-model --test vis_writers`. Each test writes the `vis_derive` fixture
model to a temp directory, derives the BDD of `Sys` and the IBD of `Sys::PowerSystem` through
the real walker, and compares the writers' output with the golden files under
`crates/syscribe-model/tests/vis_snapshots/writers/` (refresh with
`SYSCRIBE_UPDATE_SNAPSHOTS=1`). The `export-html` half of `REQ-TRS-VIS-010` (rule 1 inlining
the pinned SVG, rule 4 the placeholder) is covered by
`fully_pinned_diagram_is_drawn_inline_by_the_svg_writer` and
`diagram_without_a_picture_gets_a_placeholder` in `crates/syscribe/tests/export_html.rs`.

```gherkin
Feature: Mermaid and SVG writers off the IR (TC-TRS-VIS-010)

  Scenario: Mermaid of the derived BDD
    Given the derived BDD of the fixture package
    When it is rendered as Mermaid
    Then the text is a classDiagram matching its golden snapshot
    And every block is preceded by a %% ref: line with its qualified name

  Scenario: Mermaid of the derived IBD
    Given the derived IBD of the fixture PowerSystem
    When it is rendered as Mermaid
    Then the text is a flowchart with a subgraph per boundary and nested block, ports as small nodes, matching its snapshot

  Scenario: Mermaid click lines
    When the links closure returns a URL for two nodes
    Then exactly those two nodes get a click <id> href "<url>" _blank line, and none without links

  Scenario: SVG of the pinned derived BDD and IBD
    Given a layout pinning every node with x/y/w/h
    When it is rendered as SVG
    Then the output matches its golden snapshot, carries sysml:ref on every node group and sysml:source/sysml:target on every edge
    And an unsized boundary bounds its children

  Scenario: an unpinned node
    Given a layout that omits one node
    When it is rendered as SVG
    Then render_svg returns None

  Scenario: the REQ-TRS-LINK-002 wrapper
    When the links closure returns a URL for one node
    Then only that node's <g> is wrapped in <a xlink:href="<url>" href="<url>" target="_blank" rel="noopener">, XML-escaped
    And no <a> appears when the closure returns nothing
```
