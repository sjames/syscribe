---
id: TC-TRS-DIAG-003
type: TestCase
testLevel: L3
status: draft
name: "Verify W406/W407 are raised only for inline-SVG diagrams, not PlantUML-companion or structured diagrams."
verifies:
  - REQ-TRS-DIAG-003
---

Verify that the SVG id-consistency check (`W406`/`W407`) is scoped to diagrams
whose SVG is inline (GH #158, §8.16.7 step 3).

```gherkin
Feature: SVG id consistency applies only to inline SVG

  Scenario: a PlantUML companion diagram has no inline SVG to check
    Given Diagrams::PumlCompanion with pumlMode companion, a shapes/edges manifest
      and an image reference to its anticipated .svg
    When the tool validates the model
    Then no W406 or W407 finding names PumlCompanion.md

  Scenario: a structured layout diagram has no inline SVG to check
    Given Diagrams::Structured with a layout block and no svg fenced block
    When the tool validates the model
    Then no W406 or W407 finding names Structured.md

  Scenario: an inline SVG diagram is still checked
    Given Diagrams::InlineSvg with svgMode inline whose SVG lacks the id s-inline-missing
    When the tool validates the model
    Then a W406 finding names InlineSvg.md and s-inline-missing
    And no W406 names the id s-vehicle, which the SVG carries
```
