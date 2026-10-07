---
id: TC-TRS-SYSMLV2-094
type: TestCase
testLevel: L3
status: active
name: "Verify snapshot and timeslice occurrences map to the native portion kind."
verifies:
  - REQ-TRS-SYSMLV2-094
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_last_gaps.rs
testFunctions:
  - portion_kinds_map_to_is_portion_and_portion_kind
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_last_gaps.rs`; run with `cargo test -p syscribe-model --test sysmlv2_last_gaps`.

```gherkin
Feature: SysMLv2 metadata applications and last gaps (TC-TRS-SYSMLV2-094)

  Scenario: Snapshot and timeslice occurrences map to the native portion kind
    Given a SysMLv2 source or a native element using the construct
    When it is ingested or exported as the requirement states
    Then the observable result matches the requirement
```
