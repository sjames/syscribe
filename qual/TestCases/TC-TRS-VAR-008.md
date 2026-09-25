---
id: TC-TRS-VAR-008
type: TestCase
testLevel: L3
status: draft
name: "Verify the language server's rename re-derives Configuration inheritance over the full candidate model, so renaming a base Configuration's id that an inheriting child names in derivedFrom is not refused over the child losing its inherited selection."
verifies:
  - REQ-TRS-VAR-007
---

```gherkin
Feature: Configuration inheritance survives the language server's rename candidate (TC-TRS-VAR-008)

  Background:
    Given CONF-VR-BASE-001 (approved) selecting Features::Opt
    And CONF-VR-CHILD-001 with derivedFrom CONF-VR-BASE-001 selecting Opt only by inheritance
    And the child binding Features::Opt.gain itself (legal only while Opt is inherited)

  Scenario: the fixture validates cleanly
    When the model is validated
    Then no finding is raised

  Scenario: renaming the base id is accepted and edits base and child
    When the language server is asked to rename CONF-VR-BASE-001 to CONF-VR-BASE-002
    Then the response is a WorkspaceEdit, not a refusal
    And it edits the base's id line and the child's derivedFrom line
    And the edited child still names the new id
```
