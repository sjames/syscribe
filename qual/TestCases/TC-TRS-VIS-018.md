---
id: TC-TRS-VIS-018
type: TestCase
testLevel: L2
status: active
name: "Verify the StateMachine generator derives states with entry/do/exit compartments, initial and final pseudostates, labelled transitions from both placements and spellings, a nested machine as a container, filters and W417/W418, and that the Mermaid, SVG and PlantUML writers accept the graph."
verifies:
  - REQ-TRS-VIS-018
sourceFile: repo:crates/syscribe-model/tests/vis_derive_behaviour.rs
testFunctions:
  - derived_state_machine_of_a_statedef_matches_its_golden_ir
  - state_compartments_carry_entry_do_and_exit_actions
  - transition_labels_are_accept_guard_effect_in_either_placement_and_spelling
  - initial_feeds_the_initial_state_and_the_final_state_reaches_final
  - a_substate_typed_by_a_machine_is_a_container_with_its_own_region
  - a_transition_to_a_missing_state_draws_no_edge_and_is_left_to_w929
  - state_filters_apply_by_name_or_qualified_name_and_a_stray_entry_is_w417
  - a_partdef_subject_for_a_state_machine_is_w418_and_draws_nothing
  - the_writers_accept_a_derived_state_machine
tags:
  - diagram
  - visualisation
  - statemachine
---

Hosted integration tests in `crates/syscribe-model/tests/vis_derive_behaviour.rs`; run with
`cargo test -p syscribe-model --test vis_derive_behaviour`. Each test writes the fixture model
(`Beh::Flight`, a `StateDef` with nested and top-level transitions, entry/do/exit actions,
initial and final states and a substate typed by the `Beh::Cruise` machine) to a temp directory
and runs the real walker, validator and generator on it. The generated IR is pinned by the
golden snapshot `tests/vis_snapshots/derived/flight_sm.json`
(`SYSCRIBE_UPDATE_SNAPSHOTS=1` refreshes it).

```gherkin
Feature: the StateMachine generator derives states, pseudostates and transitions (TC-TRS-VIS-018)

  Scenario: a StateDef subject matches its golden IR
    Given a StateMachine diagram with subject Beh::Flight
    When the diagram is derived through the walker and validator
    Then no W417, W418, W402 or W403 is raised
    And the top-level states are disarmed, armed, flying and landed in declaration order
    And the IR equals the golden snapshot flight_sm.json

  Scenario: compartments carry the entry, do and exit actions
    Given the derived Flight state machine
    Then armed's compartment reads "entry / Takeoff" (string form, last segment)
    And flying's reads "do / navigate" (map form, its name) and landed's "exit / Land"
    And disarmed, which declares no action, has no compartment

  Scenario: transition labels follow accept [guard] / effect in either placement and spelling
    Given the derived Flight state machine
    Then disarmed → armed is labelled "Command [armed == false]"
    And armed → flying is labelled "Command [ready] / startTakeoff"
    And flying → landed is labelled "[altitude <= 0.1] / Land"
    And the top-level from/to/trigger transition flying → disarmed is labelled "Command" and referenced as Beh::Flight::abort
    And every edge is a Transition with a deterministic id

  Scenario: the initial pseudostate feeds the initial state and the final state reaches Final
    Given the derived Flight state machine
    Then an Initial node at the top level has an edge to disarmed
    And landed has an edge to the Final node

  Scenario: a substate typed by a machine is a container with its own region
    Given the derived Flight state machine
    Then flying contains hold and track
    And flying has its own -initial and -final pseudostates with edges to hold and from track
    And hold → track is labelled "Fix"

  Scenario: a transition to a missing state draws no edge
    Given the Flight machine's transition flying → ghost
    When the diagram is derived
    Then no edge targets ghost, the diagram raises no W417/W418, and nine edges remain

  Scenario: include and exclude filter states and flag unknown entries
    Given include [disarmed, Beh::Flight::armed, limbo]
    When the diagram is derived
    Then only disarmed and armed remain with the initial edge and disarmed → armed
    And exactly one W417 names limbo
    Given exclude [flying]
    When the diagram is derived
    Then the flying container, its region and every edge touching it are gone with no W417

  Scenario: a PartDef subject is W418 and draws nothing
    Given a StateMachine diagram whose subject is the PartDef Beh::Airframe
    When the model is validated
    Then the graph has no nodes and exactly one W418 names the PartDef type

  Scenario: the writers accept the derived graph
    Given the derived Flight state machine
    Then render_mermaid yields a stateDiagram-v2 with the composite state nested and [*] pseudostates
    And render_svg lays the graph out via the embedded ELK with state and transition classes
    And render_plantuml emits the states, their compartment lines and [*] for the pseudostates
```
