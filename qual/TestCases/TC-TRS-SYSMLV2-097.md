---
id: TC-TRS-SYSMLV2-097
type: TestCase
testLevel: L3
status: active
name: "Verify w543 and the documentation name exactly the constructs that remain unmapped after REQ-TRS-SYSMLV2-086..096."
verifies:
  - REQ-TRS-SYSMLV2-097
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_last_gaps.rs
testFunctions:
  - w543_names_exactly_the_final_unmapped_kinds
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_last_gaps.rs`; run with `cargo test -p syscribe-model --test sysmlv2_last_gaps`.

```gherkin
Feature: SysMLv2 metadata applications and last gaps (TC-TRS-SYSMLV2-097)

  Scenario: W543 and the documentation name exactly the constructs that remain unmapped after REQ-TRS-SYSMLV2-086..096
    Given a SysMLv2 source or a native element using the construct
    When it is ingested or exported as the requirement states
    Then the observable result matches the requirement
```
