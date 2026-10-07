---
id: TC-TRS-SYSMLV2-064
type: TestCase
testLevel: L3
status: active
name: "Verify standard-library recognition covers the full ScalarValues membership and the common ISQ/SI names including compound units."
verifies:
  - REQ-TRS-SYSMLV2-064
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_fidelity.rs
testFunctions:
  - scalar_values_members_raise_no_unknown_member_finding
  - compound_unit_dimensions_derive_from_the_table
  - compound_unit_is_checked_against_the_quantity_type
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_fidelity.rs`; run with `cargo test -p syscribe-model --test sysmlv2_fidelity`.

```gherkin
Feature: SysMLv2 export fidelity and standard-library awareness (TC-TRS-SYSMLV2-064)

  Scenario: Standard-library recognition covers the full ScalarValues membership and the common ISQ/SI names including compound units
    Given a model using the feature
    When it is ingested, exported or validated as the requirement states
    Then the observable result matches the requirement
```
