---
id: TC-TRS-MG-011
type: TestCase
testLevel: L3
status: draft
title: "Verify mg_variant Configuration: no E201 without featureModel when marked; E201 still fires unmarked; trade-study scores it; identity projection; MG070 on non-Configuration."
verifies:
  - REQ-TRS-MG-011
---

```gherkin
Feature: parametric-variant Configuration may omit featureModel
  Scenario: a marked Configuration without featureModel validates with no E201
    Given a Configuration with mg_variant true, a valid CONF id, title, status, parameterBindings, and no featureModel
    When the model is validated
    Then no E201 finding is produced for that Configuration

  Scenario: an unmarked Configuration without featureModel still raises E201
    Given a Configuration without mg_variant and without featureModel
    When the model is validated
    Then E201 is raised for the missing featureModel

  Scenario: trade-study scores a parametric variant from its parameterBindings
    Given a marked Configuration with parameterBindings and an mg_moe element
    When trade-study is run
    Then the configuration appears as a scored column

  Scenario: validate --config on a parametric variant projects the identity
    Given a marked Configuration with no featureModel
    When validate --config is run against it
    Then the command does not error or panic

  Scenario: mg_variant on a non-Configuration raises MG070 under the gate
    Given a PartDef marked mg_variant true
    When validate --profile magicgrid is run
    Then MG070 is raised
```
