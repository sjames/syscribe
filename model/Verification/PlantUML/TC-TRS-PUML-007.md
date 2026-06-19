---
type: TestCase
id: TC-TRS-PUML-007
name: "Batch plantuml command generates .puml files for all pumlMode:companion diagrams"
status: draft
testLevel: L2
tags:
  - diagram
  - plantuml
verifies:
  - REQ-TRS-PUML-010
---

Verifies that running `syscribe plantuml` with no positional argument operates in batch mode,
writing `.puml` files only for `Diagram` elements that carry `pumlMode: companion`, and leaving
diagrams without that field untouched.

```gherkin
Feature: Batch plantuml generates companion files for opted-in diagrams only

  Scenario: Batch run writes .puml files for companion diagrams and skips the rest
    Given a model with two Diagram elements that each have pumlMode: companion set
    And a third Diagram element that has no pumlMode field
    When syscribe -m <root> plantuml is run with no positional argument
    Then exactly two .puml files are written to their resolved output paths
    And the output paths are derived from pumlFile if present, or <stem>.puml by default
    And no .puml file is written for the Diagram element that has no pumlMode
```
