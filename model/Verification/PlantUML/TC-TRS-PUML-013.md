---
type: TestCase
id: TC-TRS-PUML-013
name: "W413 is emitted when pumlMode:companion body contains no img tag"
status: draft
testLevel: L2
tags:
  - diagram
  - plantuml
verifies:
  - REQ-TRS-PUML-030
---

Verifies that the validator emits warning `W413` when a `Diagram` element with
`pumlMode: companion` has a Markdown body that contains no `<img` tag, and that adding a
well-formed `<img>` tag suppresses the warning.

```gherkin
Feature: W413 warning when companion diagram body lacks an img tag

  Scenario: pumlMode:companion body with no img tag triggers W413
    Given a Diagram element with pumlMode: companion and diagramKind: BDD
    And the element's Markdown body contains no <img tag
    When syscribe -m <root> validate is run
    Then the output contains W413

  Scenario: pumlMode:companion body with a well-formed img tag produces no W413
    Given a Diagram element with pumlMode: companion and diagramKind: BDD
    And the element's Markdown body contains <img src="./MyDiagram.svg"/>
    When syscribe -m <root> validate is run
    Then the output does not contain W413
```
