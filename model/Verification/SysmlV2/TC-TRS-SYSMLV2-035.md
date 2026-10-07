---
id: TC-TRS-SYSMLV2-035
type: TestCase
testLevel: L3
status: active
name: "Verify a SysMLv2 use case def/use case maps to the native UseCaseDef/UseCase, reusing the case-family subject/actors/objectives lift."
verifies:
  - REQ-TRS-SYSMLV2-035
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_constraints_calcs.rs
testFunctions:
  - use_case_def_and_usage_are_mapped_with_case_fields
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_constraints_calcs.rs`; run with `cargo test -p syscribe-model --test sysmlv2_constraints_calcs`.

```gherkin
Feature: usecase mapping (TC-TRS-SYSMLV2-035)

  Scenario: ingested construct becomes a native element
    Given a sysmlSubmodel file declaring the construct
    When the tool loads the model
    Then the native element exists with the lifted fields and the construct is not counted by W543
```
