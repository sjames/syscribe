---
id: TC-TRS-SYSMLV2-020
type: TestCase
testLevel: L3
status: active
name: "Verify an ingested SysMLv2 state def/state becomes a native StateDef/State with subStates, canonical transitions, entry/do/exit actions, and the W070-W072 completeness checks apply to it unchanged."
verifies:
  - REQ-TRS-SYSMLV2-018
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_states.rs
testFunctions:
  - a_top_level_state_def_becomes_a_real_element_with_substates
  - transition_fields_use_the_canonical_source_target_accept_guard_effect_shape
  - a_nested_transition_with_no_explicit_first_clause_omits_source_implicit_from_nesting
  - entry_do_exit_action_names_lift_onto_the_substate
  - a_top_level_standalone_state_usage_becomes_its_own_element
  - dead_and_trap_states_raise_w070_and_w071
  - non_deterministic_transitions_raise_w072
  - a_clean_state_machine_raises_no_w07x_findings
tags:
  - sysmlv2
---

Hosted integration tests in `crates/syscribe-model/tests/sysmlv2_states.rs`; run with `cargo test -p syscribe-model --test sysmlv2_states`.

```gherkin
Feature: an ingested SysMLv2 state def/state becomes a native StateDef/State with subStates (TC-TRS-SYSMLV2-020)

  Scenario: a state def becomes a StateDef with inline sub-states
    Given a sysmlSubmodel file declaring a state def with nested states
    When the tool ingests the model
    Then a real StateDef exists and the nested states appear as subStates entries, not separate elements

  Scenario: transitions use the canonical schema
    Given a state def with first/accept/if/do transitions
    When the tool ingests the model
    Then transitions carry source, target, accept, guard and effect, and an implicit source is omitted

  Scenario: entry, do and exit actions lift onto the state
    Given a state with entry, do and exit actions
    When the tool ingests the model
    Then entryAction, doAction and exitAction carry the action names

  Scenario: hand-authored completeness checks apply
    Given dead, trap, non-deterministic and clean synthesized state machines
    When the tool validates the model
    Then W070, W071 and W072 fire on the defective machines and none fire on the clean one
```
