---
id: TC-TRS-VIS-016
type: TestCase
testLevel: L2
status: active
name: "Verify the embedded ELK lays the derived IBD/BDD fixtures out cleanly (no overlap, ports on borders, edges routed, labels inside), honours pins (fixed when fully pinned, interactive otherwise), lays a 200-node BDD out within the bound, draws unpinned diagrams through the CLI, MCP and export-html, and is pinned to the client's elkjs version."
verifies:
  - REQ-TRS-VIS-016
sourceFile: repo:crates/syscribe-model/tests/vis_layout.rs
testFunctions:
  - the_derived_ibd_lays_out_with_ports_on_borders_and_no_overlap
  - the_derived_bdd_lays_out_with_supertypes_above_and_compartments_inside
  - a_two_hundred_node_bdd_lays_out_within_the_bound
  - pinned_nodes_keep_their_positions_and_a_fully_pinned_graph_uses_fixed
tags:
  - diagram
  - visualisation
  - elk
---

Hosted integration tests in `crates/syscribe-model/tests/vis_layout.rs`; run with
`cargo test -p syscribe-model --test vis_layout`. Each test writes the `vis_derive` fixture
model to a temp directory, derives the IBD of `Sys::PowerSystem` or the BDD of `Sys` through
the real walker, sizes it with the approximate metrics (font-independent) and lays it out with
`vis::layout` — the vendored `elk.bundled.js` under QuickJS. The bundle-version pin is
`the_vendored_bundle_version_equals_the_frontend_lockfile_pin` in
`crates/syscribe-model/tests/vis_elk_vendor.rs`; the unpinned-diagram surfaces are
`svg_lays_out_an_unpinned_diagram_with_the_embedded_elk` (`crates/syscribe/tests/diagram_export.rs`),
`render_diagram_svg_draws_pinned_and_unpinned_diagrams_alike` (`crates/syscribe/tests/mcp_diagrams.rs`)
and `unpinned_diagram_is_laid_out_by_the_embedded_elk` / `diagram_without_a_picture_gets_a_placeholder`
(`crates/syscribe/tests/export_html.rs`); the writer side is
`svg_of_the_unpinned_derived_ibd_is_laid_out_by_elk_and_matches_its_snapshot` in `TC-TRS-VIS-010`.

```gherkin
Feature: ELK embedded in the executable (TC-TRS-VIS-016)

  Scenario: the derived IBD
    Given the derived IBD of the fixture PowerSystem with no pins
    When it is laid out by the embedded ELK
    Then no two sibling blocks overlap and every block is inside the boundary
    And every port's centre is on its parent's border within one pixel
    And both edges are routed, the connection from the engine's out port to the motor's in port
    And every label is inside its node and the blocks keep their carried size

  Scenario: the derived BDD
    Given the derived BDD of the fixture package
    When it is laid out
    Then Base sits above Engine and Motor and their inheritance edges run upward
    And each compartment sits inside its block below the name, spanning its width

  Scenario: a large diagram
    Given a synthetic BDD of 200 blocks and about 300 edges with labels
    When it is laid out
    Then the layout completes within the bound, the time is printed, and no siblings overlap

  Scenario: pins
    Given a fully pinned IBD
    Then the ELK input uses the fixed algorithm on the root and the boundary with no interactive options
    And every node keeps its parent-relative position and pinned size, labels are placed, edges are straight lines
    Given one pinned node among unpinned ones
    Then the root and the compound boundary carry the interactive strategies and the pinned node carries elk.position
    And the layout succeeds with the pinned node ordered as its position says
```
