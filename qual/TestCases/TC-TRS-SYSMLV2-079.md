---
id: TC-TRS-SYSMLV2-079
type: TestCase
testLevel: L3
status: active
name: "Verify assign referent and until loops are native syntax; the step annotation shrinks to what has no syntax."
verifies:
  - REQ-TRS-SYSMLV2-079
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_native_syntax.rs
testFunctions:
  - assign_referent_and_until_loops_are_ingested
  - referent_and_until_export_natively_and_a_dotted_target_is_the_same_value
  - value_kind_is_the_one_assign_field_that_still_needs_the_annotation
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_native_syntax.rs`; run with `cargo test -p syscribe-model --test sysmlv2_native_syntax`.

```gherkin
Feature: SysMLv2 native syntax (TC-TRS-SYSMLV2-079)

  Scenario: assign referent and until loops are native syntax; the step annotation shrinks to what has no syntax
    Given a SysMLv2 source or a native element using the construct
    When it is ingested or exported as the requirement states
    Then the observable result matches the requirement
```
