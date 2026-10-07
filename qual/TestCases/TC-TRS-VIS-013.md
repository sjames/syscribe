---
id: TC-TRS-VIS-013
type: TestCase
testLevel: L3
status: active
name: "Verify the CLI diagram toolkit is gone: its subcommands are rejected, the diagram man page documents only export, and render, plantuml and Mermaid diagrams are unaffected."
verifies:
  - REQ-TRS-VIS-013
sourceFile: repo:qual/tests/tc/TC-TRS-VIS-013.sh
tags:
  - diagram
  - visualisation
---

Black-box CLI check run by the qualification runner (`bash qual/tests/run_qual.sh
TC-TRS-VIS-013`) against the fixture model under `qual/fixtures/TC-TRS-VIS-013/model`. Since
Phase 3 (`REQ-TRS-VIS-009`) `diagram export` is the one `diagram` subcommand, so the man page
exists again and must offer nothing else.

```gherkin
Feature: the CLI diagram toolkit is removed (TC-TRS-VIS-013)

  Scenario: the retired toolkit subcommands are rejected
    When the tool runs diagram list, measure, compose, layout, seq or req against a valid model
    Then each exits non-zero, prints nothing on stdout and names the unrecognized subcommand on stderr

  Scenario: the diagram man page describes only export
    When the tool runs help diagram
    Then it exits 0, prints a SYNOPSIS that documents diagram export
    And the SYNOPSIS offers none of list, measure, compose, layout, seq, req

  Scenario: the surviving diagram commands keep their man pages
    When the tool runs help render and help plantuml
    Then each exits 0 and prints a SYNOPSIS

  Scenario: a Mermaid-kind diagram still validates
    Given a model with a diagramKind Mermaid diagram carrying a mermaid block
    When the model is validated
    Then it reports 0 errors
```
