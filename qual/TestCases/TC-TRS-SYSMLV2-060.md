---
id: TC-TRS-SYSMLV2-060
type: TestCase
testLevel: L3
status: active
name: "Verify ingestion preserves a declared name on a single-control-statement named action step."
verifies:
  - REQ-TRS-SYSMLV2-060
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_fidelity.rs
testFunctions:
  - single_statement_action_usage_ingests_as_a_named_step
  - other_nested_action_usages_stay_perform_actions
  - a_named_step_does_not_advance_the_synthesized_counter
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_fidelity.rs`; run with `cargo test -p syscribe-model --test sysmlv2_fidelity`.

```gherkin
Feature: SysMLv2 export fidelity and standard-library awareness (TC-TRS-SYSMLV2-060)

  Scenario: Ingestion preserves a declared name on a single-control-statement named action step
    Given a model using the feature
    When it is ingested, exported or validated as the requirement states
    Then the observable result matches the requirement
```
