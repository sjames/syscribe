---
id: TC-TRS-SYSMLV2-070
type: TestCase
testLevel: L3
status: active
name: "Verify action step fields with no SysML v2 text form travel in a SyscribeStep annotation: trigger and until loops."
verifies:
  - REQ-TRS-SYSMLV2-070
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_export_close.rs
testFunctions:
  - step_extension_fields_export_and_read_back_identically
  - unknown_or_non_text_extras_still_degrade
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_export_close.rs`; run with `cargo test -p syscribe-model --test sysmlv2_export_close`.

```gherkin
Feature: SysMLv2 behaviour export closure (TC-TRS-SYSMLV2-070)

  Scenario: Action step fields with no SysML v2 text form travel in a SyscribeStep annotation: trigger and until loops
    Given a model using the feature
    When it is exported or ingested as the requirement states
    Then the observable result matches the requirement
```
