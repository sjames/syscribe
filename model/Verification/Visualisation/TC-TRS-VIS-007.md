---
id: TC-TRS-VIS-007
type: TestCase
testLevel: L2
status: active
name: "Verify the bundled ELK, driven with the production layout options, lays out a fixture IBD with every port on its block's border, port labels outside, siblings disjoint, children inside their parent and every edge routed; and that pins switch ELK to interactive or fixed mode without any implicit write-back."
verifies:
  - REQ-TRS-VIS-007
sourceFile: repo:crates/syscribe-server/frontend/test/elk-layout.test.mjs
tags:
  - diagram
  - visualisation
  - elk
---

A Node script, not Rust test functions: run with `npm test` from
`crates/syscribe-server/frontend/` (after `npm install`). It loads the same `elkjs/lib/elk.bundled.js`
the browser bundle vendors, hand-builds the fixture IBD with the option values
`src/layout.ts`'s `SyscribeLayoutConfigurator` emits (graph, node and port options), runs
`elk.layout`, and asserts the geometry. A change to those option values is exercised here
without a DOM; the script exits non-zero on the first failed assertion and prints
`elk-layout: ok (...)` otherwise.

The first three scenarios are what the script asserts. The last three are properties of the
configurator and the editor established by inspection of `src/layout.ts` and `src/editor.ts`
(the option switch on `LayoutState.anyPinned`/`allPinned`, and the absence of any `PATCH`/`DELETE`
outside the drag, *Pin all* and *Auto-layout* handlers); a Node test that drives the configurator
over a pinned fixture is owed and tracked in `docs/design/visualisation.md` §10.

```gherkin
Feature: ELK layout in the browser honours ports and pins and writes nothing back (TC-TRS-VIS-007)

  Scenario: an unpinned IBD lays out cleanly
    Given a fixture IBD of a PowerSystem boundary with its own port, a battery block and a pdu block with ports, a connection edge and a binding edge
    And the graph, node and port layout options the configurator emits (layered, RIGHT, INCLUDE_CHILDREN, orthogonal routing, FIXED_SIDE, labels inside, port labels outside)
    When the bundled elkjs lays the graph out
    Then no two sibling nodes overlap
    And every child node and every node label fits inside its parent
    And both edges have routed sections

  Scenario: ports sit on their block's border
    Given the laid-out fixture
    Then all four ports have numeric x, y, width and height
    And each port's centre lies on a vertical or horizontal border of its parent
    And each port label lies outside its parent

  Scenario: the layout is the production configuration
    Given the option values in the script
    Then they equal the values SyscribeLayoutConfigurator emits for a graph, a block, a boundary and a port

  Scenario: some nodes pinned switches ELK to interactive mode
    Given a graph whose pinned set names some but not all nodes
    When the configurator applies to the root and to each compound node
    Then elk.interactive, INTERACTIVE layering and crossing minimisation and NODES_AND_EDGES model order are set
    And each pinned node carries elk.position with its pinned coordinates

  Scenario: every node pinned switches ELK to fixed
    Given a graph whose pinned set names every node
    When the configurator applies
    Then elk.algorithm is fixed on the root and on every compound node
    And a pinned w and h win over the measured size

  Scenario: an automatic layout is never written back
    Given a diagram opened and laid out by ELK
    When the user neither drags a node nor presses Pin all nor Auto-layout
    Then no PATCH or DELETE to /api/diagrams/layout is sent and the diagram file is unchanged
```
