---
id: TC-TRS-NAME-002
type: TestCase
testLevel: L3
status: draft
name: "Verify name is the universal label: E024 retired, E025 fires on any title field, FeatureDef id+name clean."
verifies:
  - REQ-TRS-NAME-002
---

```gherkin
Feature: name is the single human-readable label on every element
  Scenario: title is removed; name is the universal label
    Given a model with a Requirement carrying id + name (no title)
    And a Requirement carrying id + name + a stray title field
    And a PartDef carrying a stray title field
    And a FeatureDef carrying a FEAT-* id and a name (no title)
    And a clean PartDef (name only)
    When validate runs
    Then E024 is never emitted (the code is retired)
    And the Requirement with id + name raises no E024
    And E025 is raised naming the Requirement that declares title
    And E025 is raised naming the PartDef that declares title
    And the clean PartDef raises no E025
    And the FeatureDef with a FEAT id and a name raises neither E024 nor E025
```
