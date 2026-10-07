---
id: TC-TRS-SYSMLV2-021
type: TestCase
testLevel: L3
status: active
name: "Verify an ingested SysMLv2 action def/action becomes a native ActionDef/Action with subActions, controlNodes and successions, recursing into if/loop bodies and mapping accept/send to the native kind vocabulary."
verifies:
  - REQ-TRS-SYSMLV2-019
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_actions.rs
testFunctions:
  - a_top_level_action_def_becomes_a_real_element_with_sub_actions
  - if_action_recurses_for_real_into_then_and_else
  - loop_action_recurses_for_real_into_its_body
  - fork_and_join_become_flat_control_nodes_with_no_recoverable_body
  - a_nested_part_usage_inside_an_action_body_is_still_a_real_element
  - a_top_level_standalone_action_usage_becomes_its_own_element
  - accept_and_send_actions_map_to_the_hand_authored_kind_vocabulary
  - w080_sees_a_synthesized_action_defs_real_send_accept_sub_actions_via_a_sequence_diagram
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_actions.rs`; run with `cargo test -p syscribe-model --test sysmlv2_actions`.

```gherkin
Feature: an ingested SysMLv2 action def/action becomes a native ActionDef/Action with subActions (TC-TRS-SYSMLV2-021)

  Scenario: an action def becomes an ActionDef with subActions
    Given an action def with nested actions and successions
    When the tool ingests the model
    Then a real ActionDef exists carrying subActions and successionConnections

  Scenario: if and loop bodies recurse
    Given an action def containing an if action and a loop action
    When the tool ingests the model
    Then the then/else and loop bodies appear as real nested subActions

  Scenario: fork and join are name-only control nodes
    Given an action def with fork and join nodes
    When the tool ingests the model
    Then controlNodes lists them with no recovered body

  Scenario: accept and send use the native kind vocabulary
    Given accept and send actions and a sequence diagram over the synthesized action def
    When the tool ingests and validates the model
    Then the kinds match the hand-authored vocabulary and W080 sees the real sub-actions
```
