---
id: TC-TRS-VAL-017
type: TestCase
testLevel: L3
status: draft
name: "Verify W600 is suppressed on a Part usage whose typedBy: target already carries non-empty documentation, and still fires for a PartDef itself, a Part typed by an equally-undocumented target, and a Part with an unresolvable typedBy:."
verifies:
  - REQ-TRS-VAL-017
---

```gherkin
Feature: W600 suppression via a documented typedBy: target
  Scenario: a Part usage typed by a documented PartDef raises no W600
    Given a Part with an empty doc and typedBy: pointing at a documented PartDef
    When the model is validated
    Then no W600 is raised for that Part

  Scenario: the documented PartDef itself raises no W600
    Given that same documented PartDef
    Then no W600 is raised for it either (it has its own doc)

  Scenario: an undocumented PartDef still raises W600
    Given a PartDef with an empty doc
    When the model is validated
    Then W600 is raised for it

  Scenario: a Part typed by an equally-undocumented PartDef still raises W600
    Given a Part with an empty doc and typedBy: pointing at the undocumented PartDef
    When the model is validated
    Then W600 is raised for that Part

  Scenario: a Part with an unresolvable typedBy: still raises W600
    Given a Part with an empty doc and typedBy: pointing at a nonexistent element
    When the model is validated
    Then W600 is raised for that Part
```
