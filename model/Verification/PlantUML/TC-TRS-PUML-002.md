---
type: TestCase
id: TC-TRS-PUML-002
name: "IBD diagram produces PlantUML component diagram with nested components and flow edges"
status: draft
testLevel: L2
tags:
  - diagram
  - plantuml
verifies:
  - REQ-TRS-PUML-011
  - REQ-TRS-PUML-021
---

Verifies that the `plantuml` subcommand generates a valid PlantUML component diagram from an IBD
`Diagram` element, with a boundary rectangle wrapping nested components and flow connections
resolved through ports to their parent blocks.

```gherkin
Feature: IBD PlantUML output structure

  Scenario: IBD diagram emits nested components and resolves port connections to blocks
    Given a model with a Diagram element whose diagramKind is IBD
    And the diagram has a boundary shape containing two block shapes
    And the two blocks are connected via port shapes and a flow edge
    When syscribe -m <root> plantuml <qname> --output - is run
    Then stdout contains a rectangle block wrapping two component entries
    And stdout contains a --> connection between the two blocks
    And port shapes are not rendered as top-level entries but resolved to their parent blocks
```
