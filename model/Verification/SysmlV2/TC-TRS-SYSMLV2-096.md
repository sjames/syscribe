---
id: TC-TRS-SYSMLV2-096
type: TestCase
testLevel: L3
status: active
name: "Verify an expression payload exports unquoted when it reads back identically."
verifies:
  - REQ-TRS-SYSMLV2-096
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_last_gaps.rs
testFunctions:
  - an_expression_payload_exports_unquoted
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_last_gaps.rs`; run with `cargo test -p syscribe-model --test sysmlv2_last_gaps`.

```gherkin
Feature: SysMLv2 metadata applications and last gaps (TC-TRS-SYSMLV2-096)

  Scenario: An expression payload exports unquoted when it reads back identically
    Given a SysMLv2 source or a native element using the construct
    When it is ingested or exported as the requirement states
    Then the observable result matches the requirement
```
