---
id: TC-TRS-SYSMLV2-077
type: TestCase
testLevel: L3
status: active
name: "Verify accept and send carry their via and to targets natively."
verifies:
  - REQ-TRS-SYSMLV2-077
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_native_syntax.rs
testFunctions:
  - accept_via_and_send_via_to_are_ingested
  - via_and_to_export_as_native_statements_and_read_back
  - the_deprecated_step_annotation_is_still_ingested
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_native_syntax.rs`; run with `cargo test -p syscribe-model --test sysmlv2_native_syntax`.

```gherkin
Feature: SysMLv2 native syntax (TC-TRS-SYSMLV2-077)

  Scenario: accept and send carry their via and to targets natively
    Given a SysMLv2 source or a native element using the construct
    When it is ingested or exported as the requirement states
    Then the observable result matches the requirement
```
