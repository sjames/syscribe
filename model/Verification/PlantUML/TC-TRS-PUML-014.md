---
type: TestCase
id: TC-TRS-PUML-014
name: "W414 is emitted when pumlMode:companion .puml companion file does not exist"
status: draft
testLevel: L2
tags:
  - diagram
  - plantuml
verifies:
  - REQ-TRS-PUML-031
---

Verifies that the validator emits warning `W414` when a `Diagram` element with
`pumlMode: companion` is valid in all other respects (has `diagramKind` and an `<img>` tag) but
the resolved `.puml` companion file is absent from disk. Running `syscribe plantuml <qname>` to
generate the file must silence the warning on the next validate run.

Note: this test case itself has `status: draft`, which suppresses any W414 that the validator
would otherwise emit for this element (since no `.puml` file accompanies it in the repository).

```gherkin
Feature: W414 warning when companion .puml file is absent from disk

  Scenario: Missing .puml companion file triggers W414
    Given a Diagram element with pumlMode: companion, diagramKind: BDD, and an <img> tag in its body
    And the .puml companion file at the default path (<stem>.puml) does not exist on disk
    When syscribe -m <root> validate is run
    Then the output contains W414
    And the W414 message mentions the expected .puml file path

  Scenario: Generating the .puml file silences W414 on subsequent validation
    Given the same Diagram element as in the previous scenario
    When syscribe -m <root> plantuml <qname> is run to generate the companion file
    And syscribe -m <root> validate is run again
    Then the output does not contain W414
```
