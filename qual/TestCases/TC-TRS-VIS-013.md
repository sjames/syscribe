---
id: TC-TRS-VIS-013
type: TestCase
testLevel: L3
status: active
name: "Verify the CLI diagram toolkit is gone: diagram subcommands are rejected, its man page no longer exists, and render, plantuml and Mermaid diagrams are unaffected."
verifies:
  - REQ-TRS-VIS-013
sourceFile: repo:qual/tests/tc/TC-TRS-VIS-013.sh
tags:
  - diagram
  - visualisation
---

Black-box CLI check run by the qualification runner (`bash qual/tests/run_qual.sh
TC-TRS-VIS-013`) against the fixture model under `qual/fixtures/TC-TRS-VIS-013/model`.

```gherkin
Feature: the CLI diagram toolkit is removed (TC-TRS-VIS-013)

  Scenario: a diagram subcommand is rejected
    When the tool runs diagram list against a valid model
    Then it exits non-zero, prints nothing on stdout and names the unrecognized subcommand on stderr

  Scenario: the diagram man page no longer exists
    When the tool runs help diagram
    Then it exits non-zero and prints no SYNOPSIS

  Scenario: the surviving diagram commands keep their man pages
    When the tool runs help render and help plantuml
    Then each exits 0 and prints a SYNOPSIS

  Scenario: a Mermaid-kind diagram still validates
    Given a model with a diagramKind Mermaid diagram carrying a mermaid block
    When the model is validated
    Then it reports 0 errors
```
