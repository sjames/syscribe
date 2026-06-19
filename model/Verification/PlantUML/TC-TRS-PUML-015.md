---
id: TC-TRS-PUML-015
name: "theme config emits !theme directive in generated .puml"
type: TestCase
status: draft
testLevel: L2
verifies: [REQ-TRS-PUML-041]
tags: [diagram, plantuml, config]
---

```gherkin
Feature: [plantuml] theme config key

  Scenario: !theme directive is emitted when theme is configured
    Given a .syscribe.toml containing:
      """
      [plantuml]
      theme = "spacelab"
      """
    And a Diagram element with diagramKind: BDD
    When syscribe -m <root> plantuml <qname> --output - is run
    Then the first non-blank line of stdout is "!theme spacelab"
    And stdout does not contain "skinparam"

  Scenario: theme is ignored when style_file is also set
    Given a .syscribe.toml containing:
      """
      [plantuml]
      theme = "spacelab"
      style_file = "style/syscribe.puml"
      """
    And the file style/syscribe.puml exists in the model root
    And a Diagram element with diagramKind: BDD
    When syscribe -m <root> plantuml <qname> --output - is run
    Then stdout starts with "!include"
    And stdout does not contain "!theme"
```
