---
id: TC-TRS-SYSMLV2-086
type: TestCase
testLevel: L3
status: active
name: "Verify metadata applications on an element body are ingested into the native metadata: list."
verifies:
  - REQ-TRS-SYSMLV2-086
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_last_gaps.rs
testFunctions:
  - body_metadata_applications_lift_into_the_native_list
  - an_unresolved_metadata_type_is_the_native_e317
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_last_gaps.rs`; run with `cargo test -p syscribe-model --test sysmlv2_last_gaps`.

```gherkin
Feature: SysMLv2 metadata applications and last gaps (TC-TRS-SYSMLV2-086)

  Scenario: Metadata applications on an element body are ingested into the native metadata: list
    Given a SysMLv2 source or a native element using the construct
    When it is ingested or exported as the requirement states
    Then the observable result matches the requirement
```
