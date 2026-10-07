---
id: TC-TRS-SYSMLV2-074
type: TestCase
testLevel: L3
status: active
name: "Verify a guarded succession is ingested as a guarded successionConnections entry and exported in the same form."
verifies:
  - REQ-TRS-SYSMLV2-074
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_parser_gains.rs
testFunctions:
  - guarded_succession_is_ingested_with_its_guard
  - guarded_succession_exports_and_reads_back_identically
  - a_guard_that_is_not_an_expression_still_degrades_to_a_comment
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_parser_gains.rs`; run with `cargo test -p syscribe-model --test sysmlv2_parser_gains`.

```gherkin
Feature: SysMLv2 parser gains (TC-TRS-SYSMLV2-074)

  Scenario: A guarded succession is ingested as a guarded successionConnections entry and exported in the same form
    Given a SysMLv2 source using the construct
    When it is ingested or exported as the requirement states
    Then the observable result matches the requirement
```
