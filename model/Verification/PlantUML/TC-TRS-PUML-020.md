---
id: TC-TRS-PUML-020
name: "plantuml render generates SVG files for all pumlMode:companion diagrams"
type: TestCase
status: draft
testLevel: L2
verifies: [REQ-TRS-PUML-050]
tags: [diagram, plantuml, render]
---

```gherkin
Feature: plantuml render — batch SVG generation

  Scenario: SVG files are written next to each .puml file
    Given a model with two Diagram elements that have pumlMode: companion
    And their .puml companion files exist on disk
    And plantuml is available on PATH (or PLANTUML_JAR is set)
    When syscribe -m <root> plantuml render is run
    Then one .svg file is created next to each .puml file
    And the command prints "2 rendered, 0 failed"
    And the process exits zero

  Scenario: Diagrams without pumlMode:companion are skipped
    Given the model also contains a Diagram element without pumlMode set
    When syscribe -m <root> plantuml render is run
    Then no .puml or .svg file is written for the element without pumlMode
    And the summary counts only the companion diagrams

  Scenario: A .puml file that does not exist on disk is skipped with a warning
    Given a Diagram element with pumlMode: companion whose .puml file is absent
    When syscribe -m <root> plantuml render is run
    Then a warning is printed to stderr mentioning the missing .puml path
    And the element is not counted in the rendered total
```
