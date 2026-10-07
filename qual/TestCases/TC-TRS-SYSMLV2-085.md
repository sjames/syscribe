---
id: TC-TRS-SYSMLV2-085
type: TestCase
testLevel: L3
status: active
name: "Verify w543 and the documentation name exactly the constructs that remain unmapped."
verifies:
  - REQ-TRS-SYSMLV2-085
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_native_syntax.rs
testFunctions:
  - w543_names_exactly_the_remaining_unmapped_kinds
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_native_syntax.rs`; run with `cargo test -p syscribe-model --test sysmlv2_native_syntax`.

```gherkin
Feature: SysMLv2 native syntax (TC-TRS-SYSMLV2-085)

  Scenario: W543 and the documentation name exactly the constructs that remain unmapped
    Given a SysMLv2 source or a native element using the construct
    When it is ingested or exported as the requirement states
    Then the observable result matches the requirement
```
