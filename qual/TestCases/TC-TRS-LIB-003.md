---
id: TC-TRS-LIB-003
type: TestCase
testLevel: L3
status: draft
name: "Verify dimensional consistency (W044): quantity-type vs unit dimension must match; mismatch flagged; bare symbols handled; lenient when either side unrecognised."
verifies:
  - REQ-TRS-LIB-003
---

```gherkin
Feature: dimensional consistency between an element's quantity type and unit
  Scenario: a dimension mismatch is flagged
    Given typedBy ISQ::MassValue with unit SI::metre
    When validate runs
    Then W044 is raised naming the quantity type and the unit

  Scenario: exactly the mismatches are flagged
    Given two mismatched features (mass≠length, force≠power) among several consistent ones
    When validate runs
    Then W044 fires exactly twice

  Scenario: a consistent pair is clean (including a bare unit symbol)
    Given ISQ::MassValue+SI::kilogram, ISQ::ForceValue+SI::newton, and ISQ::MassValue+kg
    When validate runs
    Then none of them is flagged

  Scenario: lenient when either side is unrecognised
    Given ISQ::MassValue with unit USD, and ScalarValues::Real with unit SI::kilogram
    When validate runs
    Then no W044 is raised for either
```
