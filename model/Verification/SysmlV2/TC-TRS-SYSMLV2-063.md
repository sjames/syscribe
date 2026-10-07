---
id: TC-TRS-SYSMLV2-063
type: TestCase
testLevel: L3
status: active
name: "Verify round-trip tests verify cross-entry consistency of exported behaviour bodies."
verifies:
  - REQ-TRS-SYSMLV2-063
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_fidelity.rs
testFunctions:
  - cross_entry_consistency_holds_for_a_degraded_body
  - cross_entry_consistency_holds_for_the_repository_model
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_fidelity.rs`; run with `cargo test -p syscribe-model --test sysmlv2_fidelity`.

```gherkin
Feature: SysMLv2 export fidelity and standard-library awareness (TC-TRS-SYSMLV2-063)

  Scenario: Round-trip tests verify cross-entry consistency of exported behaviour bodies
    Given a model using the feature
    When it is ingested, exported or validated as the requirement states
    Then the observable result matches the requirement
```
