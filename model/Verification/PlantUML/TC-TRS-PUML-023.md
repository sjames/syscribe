---
id: TC-TRS-PUML-023
name: "plantuml command injects img tag into markdown body when absent"
type: TestCase
status: draft
testLevel: L2
verifies: [Requirements::PlantUML::REQ-TRS-PUML-054]
---

```gherkin
Feature: plantuml img tag injection

  Scenario: body has no img tag — inject on batch generation
    Given a model with a Diagram element:
      | field       | value            |
      | type        | Diagram          |
      | diagramKind | BDD              |
      | pumlMode    | companion        |
      | name        | MyBDD            |
    And the element markdown body contains no "<img" tag
    When "syscribe -m model/ plantuml" is run
    Then the companion ".puml" file is written
    And the element's ".md" file is updated to append:
      """
      <img src="./MyBDD.svg" alt="MyBDD" width="100%"/>
      """

  Scenario: body already has img tag — file is not modified
    Given a model with a Diagram element (pumlMode: companion)
    And the element markdown body already contains an "<img" tag
    When "syscribe -m model/ plantuml" is run
    Then the ".md" file is not modified

  Scenario: dry-run does not write img tag
    Given a model with a Diagram element (pumlMode: companion, body has no img tag)
    When "syscribe -m model/ plantuml --dry-run" is run
    Then no files are written or modified

  Scenario: SVG path uses pumlFile stem
    Given a Diagram element with pumlFile: "./diagrams/MyBDD.puml"
    And the element markdown body contains no "<img" tag
    When "syscribe -m model/ plantuml" is run
    Then the injected img tag src is "./diagrams/MyBDD.svg"
```
