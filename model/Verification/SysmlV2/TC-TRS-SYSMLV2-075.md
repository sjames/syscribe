---
id: TC-TRS-SYSMLV2-075
type: TestCase
testLevel: L3
status: active
name: "Verify use case include accepts qualified targets and the declaring form."
verifies:
  - REQ-TRS-SYSMLV2-075
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_parser_gains.rs
testFunctions:
  - include_accepts_qualified_and_declaring_forms
  - an_unresolved_qualified_include_is_counted_in_w543
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_parser_gains.rs`; run with `cargo test -p syscribe-model --test sysmlv2_parser_gains`.

```gherkin
Feature: SysMLv2 parser gains (TC-TRS-SYSMLV2-075)

  Scenario: Use case include accepts qualified targets and the declaring form
    Given a SysMLv2 source using the construct
    When it is ingested or exported as the requirement states
    Then the observable result matches the requirement
```
