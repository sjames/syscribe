---
id: TC-TRS-SYSMLV2-061
type: TestCase
testLevel: L3
status: active
name: "Verify export emits the named-step form for control entries whose name is not the synthesized one."
verifies:
  - REQ-TRS-SYSMLV2-061
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_fidelity.rs
testFunctions:
  - named_control_entries_export_as_named_steps_and_read_back_identically
  - export_is_stable_under_re_export_with_named_steps
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_fidelity.rs`; run with `cargo test -p syscribe-model --test sysmlv2_fidelity`.

```gherkin
Feature: SysMLv2 export fidelity and standard-library awareness (TC-TRS-SYSMLV2-061)

  Scenario: Export emits the named-step form for control entries whose name is not the synthesized one
    Given a model using the feature
    When it is ingested, exported or validated as the requirement states
    Then the observable result matches the requirement
```
