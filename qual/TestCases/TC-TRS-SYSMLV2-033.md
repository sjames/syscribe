---
id: TC-TRS-SYSMLV2-033
type: TestCase
testLevel: L3
status: active
name: "Verify a SysMLv2 constraint def/constraint maps to the native ConstraintDef/Constraint, carrying parameters, an opaque expression string, supertype/typedBy and doc."
verifies:
  - REQ-TRS-SYSMLV2-033
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_constraints_calcs.rs
testFunctions:
  - constraint_def_and_usage_are_mapped_with_expression_and_parameters
  - constraint_usage_inside_part_def_is_scoped_under_the_part
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_constraints_calcs.rs`; run with `cargo test -p syscribe-model --test sysmlv2_constraints_calcs`.

```gherkin
Feature: constraint mapping (TC-TRS-SYSMLV2-033)

  Scenario: ingested construct becomes a native element
    Given a sysmlSubmodel file declaring the construct
    When the tool loads the model
    Then the native element exists with the lifted fields and the construct is not counted by W543
```
