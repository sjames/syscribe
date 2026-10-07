---
id: TC-TRS-SYSMLV2-082
type: TestCase
testLevel: L3
status: active
name: "Verify then fork, join, decide, accept, send and if produce nodes and succession edges."
verifies:
  - REQ-TRS-SYSMLV2-082
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_native_syntax.rs
testFunctions:
  - then_control_forms_produce_nodes_and_edges
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_native_syntax.rs`; run with `cargo test -p syscribe-model --test sysmlv2_native_syntax`.

```gherkin
Feature: SysMLv2 native syntax (TC-TRS-SYSMLV2-082)

  Scenario: then fork, join, decide, accept, send and if produce nodes and succession edges
    Given a SysMLv2 source or a native element using the construct
    When it is ingested or exported as the requirement states
    Then the observable result matches the requirement
```
