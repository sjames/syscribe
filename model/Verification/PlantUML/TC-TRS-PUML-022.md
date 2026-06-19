---
id: TC-TRS-PUML-022
name: "plantuml render continues after per-file failures and summarises results"
type: TestCase
status: draft
testLevel: L2
verifies: [REQ-TRS-PUML-052, REQ-TRS-PUML-053]
tags: [diagram, plantuml, render]
---

```gherkin
Feature: plantuml render — error handling and dry-run

  Scenario: Per-file PlantUML failure does not abort the batch
    Given three .puml companion files exist
    And the second file contains a PlantUML syntax error that causes non-zero exit
    When syscribe -m <root> plantuml render is run
    Then the first and third files produce .svg output
    And stderr contains the PlantUML error output for the second file
    And the summary line reads "2 rendered, 1 failed"
    And the process exits non-zero

  Scenario: --dry-run prints paths without invoking PlantUML
    Given two Diagram elements with pumlMode: companion and existing .puml files
    When syscribe -m <root> plantuml render --dry-run is run
    Then stdout lists the two .puml paths (one per line)
    And no SVG files are created
    And no external process is spawned
    And the process exits zero
```
