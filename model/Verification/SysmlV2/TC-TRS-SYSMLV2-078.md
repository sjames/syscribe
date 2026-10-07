---
id: TC-TRS-SYSMLV2-078
type: TestCase
testLevel: L3
status: active
name: "Verify a time or change trigger on an accept is ingested and exported as native syntax."
verifies:
  - REQ-TRS-SYSMLV2-078
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_native_syntax.rs
testFunctions:
  - time_and_change_triggers_are_ingested
  - triggers_export_as_native_statements_and_read_back
  - a_trigger_beside_a_payload_keeps_the_deprecated_annotation
  - a_trigger_kind_with_no_syntax_and_no_payload_still_degrades
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_native_syntax.rs`; run with `cargo test -p syscribe-model --test sysmlv2_native_syntax`.

```gherkin
Feature: SysMLv2 native syntax (TC-TRS-SYSMLV2-078)

  Scenario: A time or change trigger on an accept is ingested and exported as native syntax
    Given a SysMLv2 source or a native element using the construct
    When it is ingested or exported as the requirement states
    Then the observable result matches the requirement
```
