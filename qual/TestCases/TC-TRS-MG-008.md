---
id: TC-TRS-MG-008
type: TestCase
testLevel: L3
status: draft
name: "Verify MoP validation: clean MoP + mopRefinedBy index; MG050 wrong host, MG051 refines missing/unresolved, MG052 target not an MoE; inert without the gate."
verifies:
  - REQ-TRS-MG-008
---

```gherkin
Feature: MagicGrid Measurement of Performance validation
  Scenario: a well-formed MoP validates clean and indexes its MoE
    Given a ConstraintDef marked mg_mop true whose mg_mop_refines names an mg_moe element
    When validate --profile magicgrid is run
    Then no MG05x finding is produced and the MoE reports the MoP under mopRefinedBy in show

  Scenario: mg_mop on the wrong host raises MG050
    Given a PartDef marked mg_mop true
    When validate --profile magicgrid is run
    Then MG050 is raised

  Scenario: a missing or unresolved mg_mop_refines raises MG051
    Given an MoP whose mg_mop_refines resolves to nothing
    When validate --profile magicgrid is run
    Then MG051 is raised

  Scenario: an mg_mop_refines target that is not an MoE raises MG052
    Given an MoP whose mg_mop_refines resolves to a plain CalculationDef without mg_moe
    When validate --profile magicgrid is run
    Then MG052 is raised

  Scenario: mg_mop fields are inert without the gate
    Given a PartDef marked mg_mop true
    When the model is validated without the magicgrid profile
    Then no MG05x finding is produced
```
