---
id: TC-TRS-SYSMLV2-087
type: TestCase
testLevel: L3
status: active
name: "Verify a metadata application with an about clause attaches to each resolvable target."
verifies:
  - REQ-TRS-SYSMLV2-087
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_last_gaps.rs
testFunctions:
  - about_targets_attach_to_the_target_or_stay_with_the_holder
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_last_gaps.rs`; run with `cargo test -p syscribe-model --test sysmlv2_last_gaps`.

```gherkin
Feature: SysMLv2 metadata applications and last gaps (TC-TRS-SYSMLV2-087)

  Scenario: A metadata application with an about clause attaches to each resolvable target
    Given a SysMLv2 source or a native element using the construct
    When it is ingested or exported as the requirement states
    Then the observable result matches the requirement
```
