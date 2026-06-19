---
type: TestCase
id: TC-TRS-PUML-008
name: "Single-element plantuml command generates .puml regardless of pumlMode"
status: draft
testLevel: L2
tags:
  - diagram
  - plantuml
verifies:
  - REQ-TRS-PUML-011
---

Verifies that when a qualified name is supplied as a positional argument to `syscribe plantuml`,
the command generates a `.puml` file unconditionally — the `pumlMode` field is not consulted in
single-element mode.

```gherkin
Feature: Single-element plantuml ignores pumlMode

  Scenario: plantuml <qname> writes a .puml file even when pumlMode is absent
    Given a model with a Diagram element that does NOT have pumlMode set
    When syscribe -m <root> plantuml <qname> is run with that element's qualified name
    Then a .puml file is written to the default path (<stem>.puml next to the .md file)
    And the process exits zero
    And no error is emitted to stderr
```
