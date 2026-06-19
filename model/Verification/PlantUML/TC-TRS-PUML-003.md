---
type: TestCase
id: TC-TRS-PUML-003
name: "StateMachine diagram produces PlantUML state diagram with [*] initial transition"
status: draft
testLevel: L2
tags:
  - diagram
  - plantuml
verifies:
  - REQ-TRS-PUML-011
  - REQ-TRS-PUML-022
---

Verifies that the `plantuml` subcommand generates a valid PlantUML state diagram from a
StateMachine `Diagram` element, including `[*]` for the initial pseudo-state and labelled
state-to-state transitions.

```gherkin
Feature: StateMachine PlantUML output structure

  Scenario: StateMachine diagram emits state declarations and initial transition
    Given a model with a Diagram element whose diagramKind is StateMachine
    And the diagram has an initial shape and two state shapes
    And the transitions include one originating from the initial shape
    When syscribe -m <root> plantuml <qname> --output - is run
    Then stdout contains state "..." as ... declarations for each state shape
    And stdout contains a [*] --> <first_state> line for the initial transition
    And stdout contains state_A --> state_B : label lines for the remaining transitions
```
