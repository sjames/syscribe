---
id: TC-TRS-SYSMLV2-018
type: TestCase
testLevel: L3
status: draft
name: "Verify a SysMLv2 allocation def (package-level or nested in a part def) synthesizes a native AllocationDef, an allocation usage typed by it resolves (no E111), and an allocation usage typed by an unknown name raises E111."
verifies:
  - REQ-TRS-SYSMLV2-029
---

```gherkin
Feature: allocation def ingestion (TC-TRS-SYSMLV2-018)

  Scenario: a package-level allocation def maps to AllocationDef
    Given a sysmlSubmodel package declaring allocation def SoftwareToHardware with a doc comment
    When the tool shows SysML2::Deploy::SoftwareToHardware
    Then it is an AllocationDef carrying the doc text

  Scenario: an allocation def nested in a part def maps to AllocationDef
    Given part def Rack { allocation def RackSlot; }
    When the tool shows SysML2::Deploy::Rack::RackSlot
    Then it is an AllocationDef

  Scenario: allocation usages typed by an ingested allocation def resolve
    Given allocation deployCtl : SoftwareToHardware and allocation slotUse : Rack::RackSlot
    When the tool validates the model
    Then no E111 names SoftwareToHardware or Rack::RackSlot

  Scenario: an allocation usage typed by an unknown name raises E111
    Given allocation broken : NoSuchAllocationDef
    When the tool validates the model
    Then E111 names NoSuchAllocationDef (the former ingested-Allocation exemption is gone)
```
