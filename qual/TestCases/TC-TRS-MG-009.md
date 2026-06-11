---
id: TC-TRS-MG-009
type: TestCase
testLevel: L3
status: draft
title: "Verify SoI marker: single SoI clean + identified in magicgrid report; MG060 wrong host, MG061 multiple SoI, MG062 also external; inert without the gate."
verifies:
  - REQ-TRS-MG-009
---

```gherkin
Feature: MagicGrid System-of-Interest boundary marker
  Scenario: a single SoI validates clean and is identified in the grid report
    Given one PartDef marked mg_soi true
    When validate --profile magicgrid is run and the magicgrid report is rendered
    Then no MG06x finding is produced and the report identifies the system of interest

  Scenario: mg_soi on a non-part raises MG060
    Given a Requirement marked mg_soi true
    When validate --profile magicgrid is run
    Then MG060 is raised

  Scenario: more than one SoI raises MG061
    Given two PartDefs each marked mg_soi true
    When validate --profile magicgrid is run
    Then MG061 is raised

  Scenario: an SoI also marked external raises MG062
    Given a PartDef marked both mg_soi true and mg_external true
    When validate --profile magicgrid is run
    Then MG062 is raised

  Scenario: mg_soi is inert without the gate
    Given two PartDefs each marked mg_soi true
    When the model is validated without the magicgrid profile
    Then no MG06x finding is produced
```
