---
type: TestCase
id: TC-TRS-PUML-010
name: "--dry-run prints file paths without writing any files"
status: draft
testLevel: L2
tags:
  - diagram
  - plantuml
verifies:
  - REQ-TRS-PUML-013
---

Verifies that the `--dry-run` flag causes the `plantuml` subcommand to print the resolved output
paths for all eligible diagrams to stdout without creating any `.puml` files on disk.

```gherkin
Feature: --dry-run prints paths without writing files

  Scenario: --dry-run lists resolved .puml paths and writes nothing to disk
    Given a model with two Diagram elements that each have pumlMode: companion set
    When syscribe -m <root> plantuml --dry-run is run
    Then stdout lists the two resolved .puml file paths, one per line
    And no .puml files are created on disk
```
