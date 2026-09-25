---
id: TC-TRS-HPLE-006
type: TestCase
testLevel: L3
status: draft
name: "Verify subConfigurations entries and cross-tier parameterBindings keys resolve through repoImports mount paths (validate and feature-check), a nearer tier's mount-path binding closes a parameter (E523, W513), and a mount path naming nothing in its peer raises E516."
verifies:
  - REQ-TRS-HPLE-001
  - REQ-TRS-HPLE-002
---

```gherkin
Feature: HPLE resolution through repoImports mount paths (TC-TRS-HPLE-006)

  Background:
    Given three product-line repos cell <- pack <- top
    And pack mounts the cell tier at Vendor::CellConfs / Vendor::CellFeatures
    And top mounts the pack tier at Supply::PackConfs

  Scenario: a mount-path subConfigurations entry and binding key resolve
    Given CONF-PACK-001 with subConfigurations Vendor::CellConfs::CONF-CELL-001
    And parameterBindings Vendor::CellFeatures::Cell.capacityAh
    When the pack model is validated
    Then no E516 and no E222 is raised

  Scenario: the mount-path binding closes the parameter
    When the pack model is validated
    Then W513 reports Features::Cell.siteCode open and does not report Features::Cell.capacityAh

  Scenario: feature-check resolves the mount-path binding key too
    When feature-check runs on the pack model
    Then no E222 is raised

  Scenario: a top tier consolidating through a mount path validates cleanly
    Given CONF-TOP-OK-001 with subConfigurations Supply::PackConfs::CONF-PACK-001 binding Features::Cell.siteCode
    When the top model is validated
    Then CONF-TOP-OK-001 raises no E516, E518, E222, E523 or W513

  Scenario: a nearer tier's mount-path binding counts as already closing the parameter
    Given CONF-TOP-DOUBLE-001 also binds Features::Cell.capacityAh
    When the top model is validated
    Then E523 names Features::Cell.capacityAh and CONF-PACK-001

  Scenario: a mount path naming nothing in its peer is dangling
    Given CONF-TOP-BAD-001 with subConfigurations Supply::PackConfs::CONF-NOPE-001
    When the top model is validated
    Then E516 names Supply::PackConfs::CONF-NOPE-001 in repo pack
```
