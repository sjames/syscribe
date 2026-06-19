---
id: TC-TRS-PUML-024
name: "base_url produces correct shape hyperlinks in generated PlantUML"
type: TestCase
status: draft
testLevel: L2
verifies: [Requirements::PlantUML::REQ-TRS-PUML-055]
---

```gherkin
Feature: base_url shape hyperlink generation

  Scenario: GitHub blob base_url produces file-path links
    Given a .syscribe.toml with:
      """
      [plantuml]
      base_url = "https://github.com/owner/repo/blob/main/model"
      """
    And a Diagram element with a shape referencing "UAV::UAVSystem"
    When "syscribe plantuml" generates the .puml file
    Then the shape annotation is:
      """
      [[https://github.com/owner/repo/blob/main/model/UAV/UAVSystem.md]]
      """

  Scenario: base_url not set — links suppressed
    Given a .syscribe.toml with no plantuml section
    When "syscribe plantuml" generates the .puml file
    Then shape lines contain no [[URL]] annotation

  Scenario: base_url empty string — links suppressed
    Given a .syscribe.toml with base_url = ""
    When "syscribe plantuml" generates the .puml file
    Then shape lines contain no [[URL]] annotation

  Scenario: deep qualified name uses slash-separated path
    Given base_url = "https://github.com/owner/repo/blob/main/model"
    And a shape referencing "Requirements::Safety::REQ-UAV-SAFE-000"
    When the .puml is generated
    Then the annotation is:
      """
      [[https://github.com/owner/repo/blob/main/model/Requirements/Safety/REQ-UAV-SAFE-000.md]]
      """
```
