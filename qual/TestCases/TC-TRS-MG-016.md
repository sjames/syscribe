---
id: TC-TRS-MG-016
type: TestCase
testLevel: L3
status: draft
title: "Verify magicgrid --svg emits a well-formed SVG of the grid (rows/pillars/SoI), -o writes a file, and the SVG works as a Diagram companion (no E402)."
verifies:
  - REQ-TRS-MG-016
---

```gherkin
Feature: magicgrid --svg renders an SVG usable as a Diagram companion
  Scenario: --svg prints a well-formed SVG to stdout
    Given a MagicGrid model
    When magicgrid --svg runs
    Then a single <svg> document is printed showing the B/W/S rows, the four pillars, and the SoI

  Scenario: -o writes the SVG to a file
    When magicgrid --svg -o <file> runs
    Then the file contains the SVG and stdout is empty

  Scenario: the SVG is a valid Diagram companion
    Given a Diagram element with svgMode: companion next to the model
    When the SVG is generated as the Diagram's same-stem companion
    Then validate raises no E402 for that Diagram
```
