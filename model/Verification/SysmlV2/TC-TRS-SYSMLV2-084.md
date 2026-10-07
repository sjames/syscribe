---
id: TC-TRS-SYSMLV2-084
type: TestCase
testLevel: L3
status: active
name: "Verify a named dependency is ingested and exported as a Dependency element."
verifies:
  - REQ-TRS-SYSMLV2-084
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_native_syntax.rs
testFunctions:
  - a_named_dependency_is_ingested_with_resolved_ends
  - a_native_dependency_exports_and_reads_back
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_native_syntax.rs`; run with `cargo test -p syscribe-model --test sysmlv2_native_syntax`.

```gherkin
Feature: SysMLv2 native syntax (TC-TRS-SYSMLV2-084)

  Scenario: A named dependency is ingested and exported as a Dependency element
    Given a SysMLv2 source or a native element using the construct
    When it is ingested or exported as the requirement states
    Then the observable result matches the requirement
```
