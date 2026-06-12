---
id: TC-TRS-MG-005
type: TestCase
testLevel: L3
status: draft
name: "Verify logical/physical layering: MG040 bad layer, MG041 unrealised logical, MG042 cross-layer coupling; allocation clears MG042; inert without the gate."
verifies:
  - REQ-TRS-MG-005
---

```gherkin
Feature: MagicGrid logical/physical layering
  Scenario: a bad layer value raises MG040 under the gate
    Given a PartDef with mg_layer subsystem
    When validate --profile magicgrid is run
    Then MG040 is raised

  Scenario: an unrealised logical element raises MG041
    Given a PartDef with mg_layer logical and no Allocation to a physical element
    When validate --profile magicgrid is run
    Then MG041 is raised

  Scenario: cross-layer coupling raises MG042
    Given a logical part whose supertype is a physical part
    When validate --profile magicgrid is run
    Then MG042 is raised

  Scenario: routing through an allocation clears MG042
    Given the same logical and physical parts related by an Allocation instead of supertype
    When validate --profile magicgrid is run
    Then MG042 is not raised

  Scenario: mg_layer is inert without the gate
    Given a PartDef with mg_layer subsystem
    When the model is validated without the magicgrid profile
    Then no MG04x finding is produced
```
