---
type: TestCase
id: TC-TRS-PUML-009
name: "--output flag writes .puml to the specified path; - writes to stdout"
status: draft
testLevel: L2
tags:
  - diagram
  - plantuml
verifies:
  - REQ-TRS-PUML-012
---

Verifies that the `--output` flag overrides the default companion file path, and that the special
value `-` directs PlantUML source to stdout without creating any file on disk.

```gherkin
Feature: --output flag controls where PlantUML source is written

  Scenario: --output <path> writes the file to the specified path
    Given a model with a Diagram element with a supported diagramKind
    When syscribe -m <root> plantuml <qname> --output /tmp/out.puml is run
    Then a file is written at /tmp/out.puml containing valid PlantUML source
    And the default companion path next to the .md file is not created

  Scenario: --output - writes PlantUML source to stdout without creating a file
    Given a model with a Diagram element with a supported diagramKind
    When syscribe -m <root> plantuml <qname> --output - is run
    Then the PlantUML source appears on stdout
    And no file is written to disk
```
