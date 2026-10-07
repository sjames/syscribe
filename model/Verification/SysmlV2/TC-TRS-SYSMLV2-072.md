---
id: TC-TRS-SYSMLV2-072
type: TestCase
testLevel: L3
status: active
name: "Verify the export summary reports how many behavioural entries were not exported."
verifies:
  - REQ-TRS-SYSMLV2-072
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_export_close.rs
testFunctions:
  - summary_counts_degraded_behaviour_entries
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_export_close.rs`; run with `cargo test -p syscribe-model --test sysmlv2_export_close`.

```gherkin
Feature: SysMLv2 behaviour export closure (TC-TRS-SYSMLV2-072)

  Scenario: The export summary reports how many behavioural entries were not exported
    Given a model using the feature
    When it is exported or ingested as the requirement states
    Then the observable result matches the requirement
```
