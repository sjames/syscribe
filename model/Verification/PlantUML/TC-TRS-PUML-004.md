---
type: TestCase
id: TC-TRS-PUML-004
name: "Sequence diagram produces PlantUML sequence diagram with actor, participant, and message arrows"
status: draft
testLevel: L2
tags:
  - diagram
  - plantuml
verifies:
  - REQ-TRS-PUML-011
  - REQ-TRS-PUML-023
---

Verifies that the `plantuml` subcommand generates a valid PlantUML sequence diagram from a
Sequence `Diagram` element. Activation and fragment shapes must be suppressed from the participant
list; message and return edges map to `->` and `-->` arrows respectively.

```gherkin
Feature: Sequence PlantUML output structure

  Scenario: Sequence diagram emits actors, participants, message arrows, and return arrows
    Given a model with a Diagram element whose diagramKind is Sequence
    And the diagram has one actor shape, two lifeline shapes, message edges, and return edges
    And the diagram frontmatter also contains activation and fragment shapes
    When syscribe -m <root> plantuml <qname> --output - is run
    Then stdout contains actor "..." as ... for the actor shape
    And stdout contains participant "..." as ... for each lifeline shape
    And stdout contains -> arrows for message edges
    And stdout contains --> arrows for return edges
    And no lines corresponding to activation or fragment shapes appear in stdout
```
