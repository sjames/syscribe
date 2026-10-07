---
id: TC-TRS-SYSMLV2-095
type: TestCase
testLevel: L3
status: active
name: "Verify a root-level alias lifts onto the submodel's anchor package."
verifies:
  - REQ-TRS-SYSMLV2-095
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_last_gaps.rs
testFunctions:
  - a_root_level_alias_lifts_onto_the_anchor_package
  - other_bare_root_members_merge_under_the_anchor
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_last_gaps.rs`; run with `cargo test -p syscribe-model --test sysmlv2_last_gaps`.

```gherkin
Feature: SysMLv2 metadata applications and last gaps (TC-TRS-SYSMLV2-095)

  Scenario: A root-level alias lifts onto the submodel's anchor package
    Given a SysMLv2 source or a native element using the construct
    When it is ingested or exported as the requirement states
    Then the observable result matches the requirement
```
