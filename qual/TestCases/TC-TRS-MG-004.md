---
id: TC-TRS-MG-004
type: TestCase
testLevel: L3
status: draft
title: "Verify MoE validation: valid MoE clean; MG030 wrong host, MG031 measures missing/unresolved, MG032 bad direction, MG033 bad bounds; inert without the gate."
verifies:
  - REQ-TRS-MG-004
---

```gherkin
Feature: MagicGrid Measure of Effectiveness validation
  Scenario: a well-formed MoE validates clean under the gate
    Given a CalculationDef marked mg_moe true with measures, unit, maximize direction, threshold 10 and objective 25
    When validate --profile magicgrid is run
    Then no MG03x finding is produced

  Scenario: mg_moe on the wrong host raises MG030
    Given a PartDef marked mg_moe true
    When validate --profile magicgrid is run
    Then MG030 is raised

  Scenario: a missing or unresolved measures raises MG031
    Given an MoE whose mg_moe_measures resolves to nothing
    When validate --profile magicgrid is run
    Then MG031 is raised

  Scenario: a bad direction raises MG032
    Given an MoE whose mg_moe_direction is bigger
    When validate --profile magicgrid is run
    Then MG032 is raised

  Scenario: inconsistent bounds raise MG033
    Given a maximize MoE with objective 5 and threshold 10
    When validate --profile magicgrid is run
    Then MG033 is raised

  Scenario: mg_moe fields are inert without the gate
    Given a PartDef marked mg_moe true
    When the model is validated without the magicgrid profile
    Then no MG03x finding is produced
```
