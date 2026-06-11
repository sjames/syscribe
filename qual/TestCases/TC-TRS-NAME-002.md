---
id: TC-TRS-NAME-002
type: TestCase
testLevel: L3
status: draft
title: "Verify the one-label-field rule: E024 (name on id-identified type), E025 (title on name-identified type), FeatureDef id+name clean."
verifies:
  - REQ-TRS-NAME-002
---

```gherkin
Feature: one label field per element, fixed by identity class
  Scenario: the wrong label field is an error, the right one is clean
    Given a model with a Requirement that wrongly declares a name field
    And a PartDef that wrongly declares a title field
    And a FeatureDef carrying both a FEAT-* id and a name (no title)
    And a clean Requirement (id + title) and a clean PartDef (name only)
    When validate runs
    Then E024 is raised naming the id-identified element with the stray name
    And E025 is raised naming the name-identified element with the stray title
    And the clean Requirement raises no E024
    And the clean PartDef raises no E025
    And the FeatureDef with a FEAT id and a name raises neither E024 nor E025
```
