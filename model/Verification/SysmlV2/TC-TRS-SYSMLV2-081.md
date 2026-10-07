---
id: TC-TRS-SYSMLV2-081
type: TestCase
testLevel: L3
status: active
name: "Verify a succession's own name and multiplicities are ingested and exported."
verifies:
  - REQ-TRS-SYSMLV2-081
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_native_syntax.rs
testFunctions:
  - succession_names_and_multiplicities_are_ingested_and_exported
  - named_and_multiplicity_successions_read_back_identically
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_native_syntax.rs`; run with `cargo test -p syscribe-model --test sysmlv2_native_syntax`.

```gherkin
Feature: SysMLv2 native syntax (TC-TRS-SYSMLV2-081)

  Scenario: A succession's own name and multiplicities are ingested and exported
    Given a SysMLv2 source or a native element using the construct
    When it is ingested or exported as the requirement states
    Then the observable result matches the requirement
```
