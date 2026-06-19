---
type: TestCase
id: TC-TRS-PUML-005
name: "Requirement diagram produces PlantUML class diagram with requirement stereotype and dependency arrows"
status: draft
testLevel: L2
tags:
  - diagram
  - plantuml
verifies:
  - REQ-TRS-PUML-011
  - REQ-TRS-PUML-024
---

Verifies that the `plantuml` subcommand generates a valid PlantUML class diagram from a
Requirement `Diagram` element, using `<<requirement>>` and `<<requirement def>>` stereotypes and
`..>` dependency arrows for `derivedFrom` and `verifies` edges.

```gherkin
Feature: Requirement diagram PlantUML output structure

  Scenario: Requirement diagram emits stereotyped class entries and dependency arrows
    Given a model with a Diagram element whose diagramKind is Requirement
    And the diagram contains Requirement and RequirementDef shapes
    And the diagram contains derivedFrom and verifies edges between those shapes
    When syscribe -m <root> plantuml <qname> --output - is run
    Then stdout contains class "..." as ... <<requirement>> for each Requirement shape
    And stdout contains class "..." as ... <<requirement def>> for each RequirementDef shape
    And stdout contains ..> arrows labelled derivedFrom for derivedFrom edges
    And stdout contains ..> arrows labelled verifies for verifies edges
```
