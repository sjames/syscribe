---
id: TC-TRS-MG-014
type: TestCase
testLevel: L3
status: draft
name: "Verify MagicGrid completeness checks: MG080 orphan need, MG081 unallocated W2 function, MG082 missing SoI, MG083 MoE without MoP; each clears when satisfied; inert without the gate."
verifies:
  - REQ-TRS-MG-014
---

```gherkin
Feature: MagicGrid completeness / coverage checks
  Scenario: an orphan B1 need raises MG080
    Given a non-draft B1 stakeholder need with no refining use case and no derived requirement
    When validate --profile magicgrid is run
    Then MG080 is raised

  Scenario: a refined or derived need clears MG080
    Given the same B1 need refined by a use case
    When validate --profile magicgrid is run
    Then MG080 is not raised

  Scenario: an unallocated W2 function raises MG081
    Given a W2 ActionDef allocated to no logical part
    When validate --profile magicgrid is run
    Then MG081 is raised

  Scenario: allocating the W2 function to a logical part clears MG081
    Given the same W2 ActionDef allocated to an mg_layer logical part
    When validate --profile magicgrid is run
    Then MG081 is not raised

  Scenario: a system context with no SoI raises MG082
    Given a model with an mg_external actor but no mg_soi
    When validate --profile magicgrid is run
    Then MG082 is raised

  Scenario: marking the SoI clears MG082
    Given the same model with one part marked mg_soi
    When validate --profile magicgrid is run
    Then MG082 is not raised

  Scenario: a MoE with no MoP raises MG083
    Given an mg_moe element that no mg_mop refines
    When validate --profile magicgrid is run
    Then MG083 is raised

  Scenario: the completeness checks are inert without the gate
    Given a model that would raise MG080-MG083 under the gate
    When the model is validated without the magicgrid profile
    Then no MG08x finding is produced
```
