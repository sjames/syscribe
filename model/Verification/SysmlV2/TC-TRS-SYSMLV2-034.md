---
id: TC-TRS-SYSMLV2-034
type: TestCase
testLevel: L3
status: active
name: "Verify a SysMLv2 calc def/calc maps to the native CalculationDef/Calculation, carrying parameters, returnType, the expression as an opaque body string and doc."
verifies:
  - REQ-TRS-SYSMLV2-034
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_constraints_calcs.rs
testFunctions:
  - calc_def_is_mapped_with_parameters_return_type_and_body
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_constraints_calcs.rs`; run with `cargo test -p syscribe-model --test sysmlv2_constraints_calcs`.

```gherkin
Feature: calc mapping (TC-TRS-SYSMLV2-034)

  Scenario: ingested construct becomes a native element
    Given a sysmlSubmodel file declaring the construct
    When the tool loads the model
    Then the native element exists with the lifted fields and the construct is not counted by W543
```
