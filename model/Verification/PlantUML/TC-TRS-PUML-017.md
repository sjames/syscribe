---
id: TC-TRS-PUML-017
name: "Built-in skinparam block is used when no plantuml config is present"
type: TestCase
status: draft
testLevel: L2
verifies: [REQ-TRS-PUML-040]
tags: [diagram, plantuml, config]
---

```gherkin
Feature: [plantuml] fallback to built-in skinparams

  Scenario: Built-in skinparams are emitted when [plantuml] section is absent
    Given a .syscribe.toml that has no [plantuml] section
    And a Diagram element with diagramKind: BDD
    When syscribe -m <root> plantuml <qname> --output - is run
    Then stdout contains "skinparam"
    And stdout does not contain "!theme"
    And stdout does not contain "!include"

  Scenario: Built-in skinparams are emitted when [plantuml] section is empty
    Given a .syscribe.toml containing:
      """
      [plantuml]
      """
    And a Diagram element with diagramKind: StateMachine
    When syscribe -m <root> plantuml <qname> --output - is run
    Then stdout contains a state diagram with no !theme or !include preamble
```
