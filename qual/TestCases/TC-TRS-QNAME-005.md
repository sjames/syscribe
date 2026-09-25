---
id: TC-TRS-QNAME-005
type: TestCase
testLevel: L3
status: draft
name: "Verify a qualifiedName: override never changes the path-derived qualified name and raises W049 when it differs."
verifies:
  - REQ-TRS-QNAME-005
---

Verify that `qualifiedName:` is not an identity override (GH #160, §3.1/§4.5).

```gherkin
Feature: qualifiedName is not an identity override

  Scenario: a differing qualifiedName is reported and ignored
    Given the PartDef at Arch/Pump.md declaring qualifiedName Other::Pump
    When the tool validates the model
    Then exactly one W049 finding is emitted, naming Pump.md, Other::Pump and Arch::Pump
    And show Arch::Pump finds the element
    And show Other::Pump finds nothing

  Scenario: a qualifiedName equal to the path-derived name is harmless
    Given the PartDef at Arch/Valve.md declaring qualifiedName Arch::Valve
    When the tool validates the model
    Then no W049 finding names Valve.md
```
