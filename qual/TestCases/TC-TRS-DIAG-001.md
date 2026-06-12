---
id: TC-TRS-DIAG-001
type: TestCase
testLevel: L3
status: draft
name: "Verify that E400–E402 and W400–W412 are emitted for Diagram element validation conditions."
verifies:
  - REQ-TRS-DIAG-001
---

Verify that the tool enforces all Diagram-element validation rules by running the binary against minimal model fixtures that each trigger exactly one code.

```gherkin
Feature: Diagram element validation rules

  Scenario: W400 — missing diagramKind
    Given a Diagram element with no diagramKind field and no svgMode field
    When the tool validates the model
    Then a W400 finding is emitted

  Scenario: E400 — Mermaid diagramKind with no mermaid fenced block
    Given a Diagram element with diagramKind: Mermaid
    And the body contains plain text but no ```mermaid fenced block
    When the tool validates the model
    Then an E400 finding is emitted

  Scenario: E401 — PlantUML diagramKind with no plantuml fenced block
    Given a Diagram element with diagramKind: PlantUML
    And the body contains plain text but no ```plantuml fenced block
    When the tool validates the model
    Then an E401 finding is emitted

  Scenario: E402 — companion svgMode with missing SVG file
    Given a Diagram element with svgMode: companion
    And no companion SVG file exists on disk
    When the tool validates the model
    Then an E402 finding is emitted

  Scenario: W401 — subject does not resolve
    Given a Diagram element with subject: NonExistentElement
    When the tool validates the model
    Then a W401 finding is emitted

  Scenario: W402 — shapes ref does not resolve
    Given a Diagram element with a shapes entry whose ref is NonExistent::Element
    When the tool validates the model
    Then a W402 finding is emitted

  Scenario: W403 — edge references undefined shape id
    Given a Diagram element with shapes defining id s1
    And an edge whose source references undefined-shape
    When the tool validates the model
    Then a W403 finding is emitted

  Scenario: W405a — svgMode companion with no img tag in body
    Given a Diagram element with svgMode: companion and a companion SVG file present
    And the body contains no <img tag
    When the tool validates the model
    Then a W405 finding is emitted

  Scenario: W405b — svgMode inline with no svg fenced block
    Given a Diagram element with svgMode: inline
    And the body contains no ```svg fenced block
    When the tool validates the model
    Then a W405 finding is emitted

  Scenario: W406 — frontmatter shape id not present in inline SVG
    Given a Diagram element in inline svgMode
    And shapes declares id s-box
    And the inline SVG body has no element with id="s-box"
    When the tool validates the model
    Then a W406 finding is emitted

  Scenario: W407 — inline SVG id not present in frontmatter shapes
    Given a Diagram element in inline svgMode
    And the inline SVG body has an element with id="orphan"
    And no frontmatter shape or edge declares id orphan
    When the tool validates the model
    Then a W407 finding is emitted

  Scenario: W408 — Mermaid %% ref annotation does not resolve
    Given a Diagram element with diagramKind: Mermaid
    And the mermaid block contains %% ref: NoSuchElement
    When the tool validates the model
    Then a W408 finding is emitted

  Scenario: W409 — Mermaid diagram has no %% ref annotations
    Given a Diagram element with diagramKind: Mermaid
    And the mermaid block contains no %% ref: annotations
    When the tool validates the model
    Then a W409 finding is emitted

  Scenario: W410 — Mermaid %% link annotation does not resolve
    Given a Diagram element with diagramKind: Mermaid
    And the mermaid block contains %% link: nodeA NoSuchElement
    When the tool validates the model
    Then a W410 finding is emitted

  Scenario: W411 — shapes link does not resolve
    Given a Diagram element with a shapes entry whose link is NoSuchElement
    When the tool validates the model
    Then a W411 finding is emitted

  Scenario: W412 — SVG href does not resolve to a model element file
    Given a Diagram element with an inline SVG block
    And the SVG contains href="./NoSuchFile.md"
    When the tool validates the model
    Then a W412 finding is emitted
```
