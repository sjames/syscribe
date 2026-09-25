---
id: TC-TRS-ALLOC-002
type: TestCase
testLevel: L3
status: draft
name: "Verify E314 and W034 consume the §12.9 unified allocation-edge set (allocatedTo, features-form and top-level Allocation elements, legacy authored allocatedFrom), and the legacy form reaches matrix --allocations and W503."
verifies:
  - REQ-TRS-TRACE-008
  - REQ-TRS-ALLOC-001
---

```gherkin
Feature: every allocation form counts for E314 and W034 (GH #131)
  Scenario: allocatedTo on the deployment package clears E314
    Given a deployment package holding allocatedTo: a hardware element
    When the model is validated
    Then no E314 is raised and the exit code is zero

  Scenario: a features-form Allocation element clears E314
    Given an Allocation element whose features: entry allocates the package to a hardware element
    When the model is validated
    Then no E314 is raised

  Scenario: a legacy authored allocatedFrom on the hardware target clears E314
    Given a hardware element holding allocatedFrom: [the deployment package]
    When the model is validated
    Then no E314 is raised

  Scenario: an allocation to a software element still raises E314
    Given a deployment package whose only allocation targets a software element
    When the model is validated
    Then E314 is raised

  Scenario: W034 sees the sources of standalone Allocation elements
    Given two Allocation elements placing an ASIL D and an ASIL B function on one ECU without an FFI argument
    When the model is validated
    Then W034 names the shared ECU and both functions

  Scenario: a legacy authored allocatedFrom reaches the matrix and W503
    Given a PartDef holding allocatedFrom: [an ActionDef]
    When matrix --allocations is run
    Then the ActionDef → PartDef edge is marked
    And the same edge also declared by allocatedTo on the ActionDef raises W503 naming both forms
```
