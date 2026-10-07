---
id: TC-TRS-SYSMLV2-083
type: TestCase
testLevel: L3
status: active
name: "Verify occurrence, event occurrence, individual definitions and occurrence usages are ingested and exported."
verifies:
  - REQ-TRS-SYSMLV2-083
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_native_syntax.rs
testFunctions:
  - occurrences_and_individuals_are_native_elements
  - native_occurrence_types_export_and_read_back
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_native_syntax.rs`; run with `cargo test -p syscribe-model --test sysmlv2_native_syntax`.

```gherkin
Feature: SysMLv2 native syntax (TC-TRS-SYSMLV2-083)

  Scenario: occurrence, event occurrence, individual definitions and occurrence usages are ingested and exported
    Given a SysMLv2 source or a native element using the construct
    When it is ingested or exported as the requirement states
    Then the observable result matches the requirement
```
