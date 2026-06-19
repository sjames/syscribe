---
type: TestCase
id: TC-TRS-PUML-012
name: "E404 is emitted when pumlMode:companion is set without diagramKind"
status: draft
testLevel: L2
tags:
  - diagram
  - plantuml
verifies:
  - REQ-TRS-PUML-033
---

Verifies that the validator emits error `E404` when a `Diagram` element sets
`pumlMode: companion` but omits the `diagramKind` field, which is required for PlantUML
generation to know which output dialect to produce.

```gherkin
Feature: E404 validation when diagramKind is missing alongside pumlMode:companion

  Scenario: pumlMode:companion without diagramKind triggers E404
    Given a Diagram element with pumlMode: companion
    And the same Diagram element has no diagramKind field
    When syscribe -m <root> validate is run
    Then the output contains E404
    And the E404 message mentions diagramKind
```
