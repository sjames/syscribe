---
id: TC-TRS-SYSMLV2-057
type: TestCase
testLevel: L3
status: active
name: "Verify export-sysml emits state def/state bodies that ingestion reads back: entry/do/exit, substates, initial/final markers and transitions with accept, guard and effect."
verifies:
  - REQ-TRS-SYSMLV2-057
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_behavior_export.rs
testFunctions:
  - state_def_body_round_trips_through_ingestion
  - state_usage_round_trips
  - nested_transition_without_explicit_source_round_trips
  - accept_via_and_time_trigger_round_trip
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_behavior_export.rs`; run with `cargo test -p syscribe-model --test sysmlv2_behavior_export`.

```gherkin
Feature: SysMLv2 behaviour export and ingestion accounting (TC-TRS-SYSMLV2-057)

  Scenario: export-sysml emits state def/state bodies that ingestion reads back: entry/do/exit, substates, initial/final markers and transitions with accept, guard and effect
    Given a model using the feature
    When it is ingested, exported or reported as the requirement states
    Then the observable result matches the requirement
```
