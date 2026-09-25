---
id: TC-TRS-SYSMLV2-019
type: TestCase
testLevel: L3
status: draft
name: "Verify an ingested SysMLv2 allocation usage's allocate clause is lifted into allocatedFrom/allocatedTo with full-model endpoint resolution, feeds matrix --allocations and E314, truncates an unresolvable chain tail with W542, and reports unresolvable endpoints with E502/E503."
verifies:
  - REQ-TRS-SYSMLV2-029
---

```gherkin
Feature: allocation usage allocate-clause lifting (TC-TRS-SYSMLV2-019)

  Scenario: a cross-package allocate clause becomes allocatedFrom/allocatedTo
    Given allocation deploy : SoftwareToHardware allocate Arch::SwPackage to Arch::Board inside package Deploy
    When the tool shows SysML2::Deploy::deploy
    Then allocatedFrom is Arch::SwPackage and allocatedTo is Arch::Board

  Scenario: the ingested allocation feeds the unified allocation set
    Given Arch::SwPackage is an isDeploymentPackage software PartDef with no native allocation
    When the tool validates the model and prints matrix --allocations
    Then no E314 is raised for SwPackage and the SwPackage -> Board edge is listed

  Scenario: feature chains resolve through the heads' part-def types
    Given allocation chained allocate sys.ctl to board.mcu where sys : Controller { part ctl } and board : Hw { part mcu }
    When the tool shows SysML2::Deploy::chained
    Then allocatedFrom is SysML2::Deploy::Controller::ctl and allocatedTo is SysML2::Deploy::Hw::mcu

  Scenario: an unresolvable chain tail is truncated with W542
    Given allocation truncated allocate sys.nosuch to board
    When the tool validates the model
    Then W542 names the endpoint sys.nosuch and allocatedFrom is SysML2::Deploy::sys

  Scenario: endpoints that resolve nowhere are reported
    Given allocation dangling allocate NoSuchSource to NoSuchTarget
    When the tool validates the model
    Then E502 names NoSuchSource and E503 names NoSuchTarget
```
