---
id: TC-TRS-PUML-019
name: "Custom base_url and empty base_url are respected in shape URL annotations"
type: TestCase
status: draft
testLevel: L2
verifies: [REQ-TRS-PUML-043, REQ-TRS-PUML-044]
tags: [diagram, plantuml, config]
---

```gherkin
Feature: PlantUML clickable element links — custom and suppressed base_url

  Scenario: Custom base_url is used in generated URL annotations
    Given a .syscribe.toml containing:
      """
      [plantuml]
      base_url = "https://syscribe.example.com"
      """
    And a Diagram element with diagramKind: BDD containing a shape with ref: UAV::UAVSystem
    When syscribe -m <root> plantuml <qname> --output - is run
    Then each class declaration contains "[[https://syscribe.example.com/ui/detail/UAV::UAVSystem]]"
    And no declaration contains "localhost"

  Scenario: Empty base_url suppresses all URL annotations
    Given a .syscribe.toml containing:
      """
      [plantuml]
      base_url = ""
      """
    And a Diagram element with diagramKind: BDD containing shapes with refs
    When syscribe -m <root> plantuml <qname> --output - is run
    Then no declaration line contains "[["
```
