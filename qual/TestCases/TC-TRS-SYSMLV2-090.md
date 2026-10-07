---
id: TC-TRS-SYSMLV2-090
type: TestCase
testLevel: L3
status: active
name: "Verify control node parameter bodies are ingested and exported."
verifies:
  - REQ-TRS-SYSMLV2-090
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_last_gaps.rs
testFunctions:
  - control_node_parameters_are_ingested_and_exported
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_last_gaps.rs`; run with `cargo test -p syscribe-model --test sysmlv2_last_gaps`.

```gherkin
Feature: SysMLv2 metadata applications and last gaps (TC-TRS-SYSMLV2-090)

  Scenario: Control node parameter bodies are ingested and exported
    Given a SysMLv2 source or a native element using the construct
    When it is ingested or exported as the requirement states
    Then the observable result matches the requirement
```
