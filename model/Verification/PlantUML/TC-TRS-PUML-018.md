---
id: TC-TRS-PUML-018
name: "Default base_url emits localhost element links on all shape declarations"
type: TestCase
status: draft
testLevel: L2
verifies: [REQ-TRS-PUML-043, REQ-TRS-PUML-044]
tags: [diagram, plantuml, config]
---

```gherkin
Feature: PlantUML clickable element links — default base_url

  Scenario: BDD shape declarations include [[http://localhost:3000/ui/detail/<ref>]] by default
    Given a .syscribe.toml with no [plantuml] section (or no base_url key)
    And a Diagram element with diagramKind: BDD containing a shape with ref: UAV::UAVSystem
    When syscribe -m <root> plantuml <qname> --output - is run
    Then each class declaration line contains "[[http://localhost:3000/ui/detail/UAV::UAVSystem]]"

  Scenario: IBD boundary and block declarations include URL annotations
    Given a .syscribe.toml with no base_url key
    And a Diagram element with diagramKind: IBD containing boundary and block shapes with refs
    When syscribe -m <root> plantuml <qname> --output - is run
    Then the rectangle declaration line contains "[[http://localhost:3000/ui/detail/<ref>]]"
    And each component declaration line contains "[[http://localhost:3000/ui/detail/<ref>]]"

  Scenario: StateMachine state declarations include URL annotations
    Given a Diagram element with diagramKind: StateMachine containing state shapes with refs
    When syscribe -m <root> plantuml <qname> --output - is run
    Then each state declaration line contains "[[http://localhost:3000/ui/detail/<ref>]]"
    And the initial pseudo-state ([*]) line has no URL annotation

  Scenario: Sequence participant declarations include URL annotations
    Given a Diagram element with diagramKind: Sequence with actor and lifeline shapes with refs
    When syscribe -m <root> plantuml <qname> --output - is run
    Then each actor and participant declaration line contains "[[http://localhost:3000/ui/detail/<ref>]]"
```
