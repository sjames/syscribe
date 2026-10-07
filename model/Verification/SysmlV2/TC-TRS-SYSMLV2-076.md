---
id: TC-TRS-SYSMLV2-076
type: TestCase
testLevel: L3
status: active
name: "Verify a bare package-level attribute, port or item is read as the usage it is, with its value and unit."
verifies:
  - REQ-TRS-SYSMLV2-076
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_parser_gains.rs
testFunctions:
  - bare_package_level_usages_are_usages_with_values_and_units
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_parser_gains.rs`; run with `cargo test -p syscribe-model --test sysmlv2_parser_gains`.

```gherkin
Feature: SysMLv2 parser gains (TC-TRS-SYSMLV2-076)

  Scenario: A bare package-level attribute, port or item is read as the usage it is, with its value and unit
    Given a SysMLv2 source using the construct
    When it is ingested or exported as the requirement states
    Then the observable result matches the requirement
```
