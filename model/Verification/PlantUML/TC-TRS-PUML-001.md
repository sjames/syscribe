---
type: TestCase
id: TC-TRS-PUML-001
name: "BDD diagram produces PlantUML class diagram with block stereotypes and edge connectors"
status: draft
testLevel: L2
tags:
  - diagram
  - plantuml
verifies:
  - REQ-TRS-PUML-011
  - REQ-TRS-PUML-020
---

Verifies that the `plantuml` subcommand generates a valid PlantUML class diagram from a BDD
`Diagram` element, including `<<part def>>` stereotypes and composition connectors.

```gherkin
Feature: BDD PlantUML output structure

  Scenario: BDD diagram emits class declarations and composition connectors
    Given a model with a Diagram element whose diagramKind is BDD
    And the diagram has two PartDef shapes connected by a composition edge
    When syscribe -m <root> plantuml <qname> --output - is run
    Then stdout starts with @startuml
    And stdout contains class declarations with <<part def>> stereotype for each PartDef shape
    And stdout contains a *-- connector representing the composition edge
    And stdout ends with @enduml
```
