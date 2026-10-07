---
id: TC-TRS-VIS-010
type: TestCase
testLevel: L2
status: active
name: "Verify the Mermaid and static SVG writers are pure functions of the IR: golden snapshots for the derived BDD/IBD fixture pinned and unpinned, a %% ref: per Mermaid node, sysml:ref per SVG node and sysml:source/target per edge, an unpinned graph laid out by the embedded ELK, refusal only of an empty graph, and the REQ-TRS-LINK-002 hyperlink wrapper from the links closure."
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
  - svg_of_the_unpinned_derived_ibd_is_laid_out_by_elk_and_matches_its_snapshot
  - svg_wraps_linked_nodes_in_the_req_trs_link_002_anchor_and_nothing_else
tags:
  - diagram
  - visualisation
---

Hosted integration tests in `crates/syscribe-model/tests/vis_writers.rs`; run with
`cargo test -p syscribe-model --test vis_writers`. Each test writes the `vis_derive` fixture
model to a temp directory, derives the BDD of `Sys` and the IBD of `Sys::PowerSystem` through
the real walker, sizes it with the approximate metrics (so the snapshots do not depend on the
fonts installed), and compares the writers' output with the golden files under
`crates/syscribe-model/tests/vis_snapshots/writers/` (refresh with
`SYSCRIBE_UPDATE_SNAPSHOTS=1`). The `export-html` half of `REQ-TRS-VIS-010` (rule 1 inlining
the pinned and the laid-out SVG, rule 4 the placeholder) is covered by
`fully_pinned_diagram_is_drawn_inline_by_the_svg_writer`,
`unpinned_diagram_is_laid_out_by_the_embedded_elk` and
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

  Scenario: an unpinned graph
    Given the derived IBD with no layout at all
    When it is rendered as SVG
    Then the embedded ELK lays it out and the output matches its golden snapshot, every node grouped, every edge routed, edge labels placed
    Given a layout that omits one node
    Then the graph still draws, around the pins
    Given a derived diagram whose subject yields no shapes
    Then render_svg is SvgError::Empty

  Scenario: the REQ-TRS-LINK-002 wrapper
    When the links closure returns a URL for one node
    Then only that node's <g> is wrapped in <a xlink:href="<url>" href="<url>" target="_blank" rel="noopener">, XML-escaped
    And no <a> appears when the closure returns nothing
```
