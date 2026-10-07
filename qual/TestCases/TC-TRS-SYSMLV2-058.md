---
id: TC-TRS-SYSMLV2-058
type: TestCase
testLevel: L3
status: active
name: "Verify Behaviour that ingestion would not read back identically is exported as a comment, never as text that re-ingests differently."
verifies:
  - REQ-TRS-SYSMLV2-058
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_behavior_export.rs
testFunctions:
  - unrepresentable_action_entries_become_comments_and_the_rest_round_trips
  - unrepresentable_state_entries_become_comments_and_the_rest_round_trips
  - degraded_behaviour_text_still_parses
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_behavior_export.rs`; run with `cargo test -p syscribe-model --test sysmlv2_behavior_export`.

```gherkin
Feature: SysMLv2 behaviour export and ingestion accounting (TC-TRS-SYSMLV2-058)

  Scenario: Behaviour that ingestion would not read back identically is exported as a comment, never as text that re-ingests differently
    Given a model using the feature
    When it is ingested, exported or reported as the requirement states
    Then the observable result matches the requirement
```
