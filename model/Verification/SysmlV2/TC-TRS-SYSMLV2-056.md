---
id: TC-TRS-SYSMLV2-056
type: TestCase
testLevel: L3
status: active
name: "Verify export-sysml emits action def/action bodies that ingestion reads back: sub-actions, control nodes, successions and control-flow constructs."
verifies:
  - REQ-TRS-SYSMLV2-056
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_behavior_export.rs
testFunctions:
  - perform_sub_actions_round_trip
  - accept_and_send_round_trip
  - assign_round_trips
  - while_loop_and_loop_round_trip
  - for_loop_round_trips
  - if_else_round_trips_including_nested
  - terminate_round_trips
  - control_nodes_and_successions_round_trip
  - action_usage_body_round_trips
  - action_def_body_round_trips_through_ingestion
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_behavior_export.rs`; run with `cargo test -p syscribe-model --test sysmlv2_behavior_export`.

```gherkin
Feature: SysMLv2 behaviour export and ingestion accounting (TC-TRS-SYSMLV2-056)

  Scenario: export-sysml emits action def/action bodies that ingestion reads back: sub-actions, control nodes, successions and control-flow constructs
    Given a model using the feature
    When it is ingested, exported or reported as the requirement states
    Then the observable result matches the requirement
```
