---
id: TC-TRS-SYSMLV2-100
type: TestCase
testLevel: L3
status: active
name: "Verify a package-level calc usage shape maps to a Calculation and a qualified-name package is counted once in W543 with its members not ingested."
verifies:
  - REQ-TRS-SYSMLV2-100
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_audit_gaps.rs
testFunctions:
  - a_package_level_calc_usage_shape_maps_to_a_calculation
  - a_qualified_name_package_is_counted_once_and_its_members_are_not_ingested
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_audit_gaps.rs`; run with `cargo test -p syscribe-model --test sysmlv2_audit_gaps`.

```gherkin
Feature: SysMLv2 package-level members are ingested or counted (TC-TRS-SYSMLV2-100)

  Scenario: A package-level calc usage shape maps to a Calculation
    Given package P { calc estimate [1]; calc def Est; }
    When the submodel is ingested
    Then P::estimate is a Calculation, P::Est a CalculationDef, and W543 is not raised

  Scenario: A qualified-name package is counted once and its members are not ingested
    Given package A::B { part def Y; alias N for X; actor Z; package Inner { part def W; } } and library package L::M { part def V; } beside package P { part def X; }
    When the submodel is ingested
    Then P::X exists, no element named Y, W or V exists, and W543 reports exactly "qualified package x2" with no alias or actor count
```
