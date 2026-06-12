---
id: TC-TRS-OUT-010
type: TestCase
testLevel: L3
status: draft
name: "Verify the element-rooted connectivity subgraph export (text tree, JSON nodes/edges, styled DOT, whole-model dump, depth bound)."
verifies:
  - REQ-TRS-OUT-010
---

Verify that `connectivity` walks the model graph outward from a chosen root element, surfacing the connection wiring as edges, and renders the reachable subgraph as a text tree, a JSON `{nodes, edges}` document, and styled Graphviz DOT. The model-root element dumps the whole model; `--depth` bounds the walk; an unknown root exits non-zero.

```gherkin
Feature: Element-rooted connectivity subgraph export

  Scenario: Text output names both wired sub-parts
    Given a parent PartDef with two sub-part features wired by a connection
    When connectivity is invoked on the parent
    Then the text tree is rooted at the parent and names both sub-parts

  Scenario: JSON exposes nodes and a connection edge between the sub-parts
    Given the same model
    When connectivity --format json is invoked on the parent
    Then the output is valid JSON carrying nodes and edges
    And it includes a connection-kind edge between the two sub-parts

  Scenario: DOT output is styled Graphviz
    Given the same model
    When connectivity --format dot is invoked on the parent
    Then the output contains digraph and shape/peripheries styling attributes

  Scenario: The model-root element dumps the whole model
    Given any model
    When connectivity is invoked on the model-root element
    Then every model element is reachable in the output

  Scenario: Depth bounds the walk
    Given the parent and its sub-parts
    When connectivity --depth 0 is invoked on the parent
    Then only the parent appears and the sub-parts do not

  Scenario: Unknown root exits non-zero
    Given any model
    When connectivity is invoked on a non-existent element
    Then the command exits non-zero
```
