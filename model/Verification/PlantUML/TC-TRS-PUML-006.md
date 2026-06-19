---
type: TestCase
id: TC-TRS-PUML-006
name: "Mermaid and unknown diagramKind values are skipped with a stderr warning"
status: draft
testLevel: L2
tags:
  - diagram
  - plantuml
verifies:
  - REQ-TRS-PUML-025
---

Verifies that when a `Diagram` element carries a `diagramKind` value for which no PlantUML
mapping exists (including `Mermaid`), the `plantuml` subcommand exits non-zero and emits a
descriptive warning to stderr.

```gherkin
Feature: Unsupported diagramKind is rejected gracefully

  Scenario: Mermaid diagramKind causes non-zero exit with a stderr warning
    Given a model with a Diagram element whose diagramKind is Mermaid
    When syscribe -m <root> plantuml <qname> is run
    Then the process exits with a non-zero exit code
    And stderr contains the qualified name of the diagram
    And stderr contains the word Mermaid
    And stderr contains the phrase no PlantUML mapping
```
