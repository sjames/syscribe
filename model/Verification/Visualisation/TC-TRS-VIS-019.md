---
id: TC-TRS-VIS-019
type: TestCase
testLevel: L2
status: active
name: "Verify the Action generator derives stereotyped steps with compartments, an if/else as decision, branches and merge, a loop as a container, fork/join control nodes, successions and flows, initial/final only when successions exist, filters and W417/W418, and that the Mermaid and SVG writers accept the graph while PlantUML declines it."
verifies:
  - REQ-TRS-VIS-019
sourceFile: repo:crates/syscribe-model/tests/vis_derive_behaviour.rs
testFunctions:
  - derived_action_of_an_actiondef_matches_its_golden_ir
  - action_steps_carry_their_kind_stereotype_and_compartment_lines
  - an_if_action_is_a_decision_with_then_else_branches_closed_by_a_merge
  - a_loop_action_is_a_container_holding_its_body_in_order
  - control_nodes_successions_and_flows_come_from_the_subjects_lists
  - initial_and_final_bracket_the_flow_only_when_successions_exist
  - action_filters_apply_to_steps_and_control_nodes_and_a_stray_entry_is_w417
  - a_partdef_subject_for_an_action_diagram_is_w418_and_draws_nothing
  - the_writers_accept_a_derived_action_diagram
tags:
  - diagram
  - visualisation
  - action
---

Hosted integration tests in `crates/syscribe-model/tests/vis_derive_behaviour.rs`; run with
`cargo test -p syscribe-model --test vis_derive_behaviour`. Each test writes the fixture model
(`Beh::Mission`, an `ActionDef` with perform/send/accept steps, an if/else, a for-loop with a
body, fork and join control nodes, successions and a flow; `Beh::Unordered` with no
successions) to a temp directory and runs the real walker, validator and generator on it. The
generated IR is pinned by the golden snapshot `tests/vis_snapshots/derived/mission_action.json`
(`SYSCRIBE_UPDATE_SNAPSHOTS=1` refreshes it).

```gherkin
Feature: the Action generator derives steps, control nodes and successions (TC-TRS-VIS-019)

  Scenario: an ActionDef subject matches its golden IR
    Given an Action diagram with subject Beh::Mission
    When the diagram is derived through the walker and validator
    Then no W400, W417, W418, W402 or W403 is raised
    And the top-level steps are takeoff, abort, proceed, navigate and land with perform/send/perform/loop/perform stereotypes
    And the IR equals the golden snapshot mission_action.json

  Scenario: steps carry their kind stereotype and compartment lines
    Given the derived Mission action
    Then takeoff's compartment reads ": Takeoff"
    And abort is a send step with "send Command" and "via ctrlOut"
    And awaitArrival is an accept step with "accept Fix" and "when near(wp)"
    And the bare action advance has no compartment

  Scenario: an IfAction is a decision with then/else branches closed by a merge
    Given the derived Mission action
    Then checkWeather is a Decision labelled "wind > 12" with a Merge
    And [then] leads to abort and [else] to proceed, both succeeding to the merge
    And takeoff → checkWeather enters at the decision and the merge → navigate edge is labelled "[ok]"

  Scenario: a LoopAction is a container holding its body in order
    Given the derived Mission action
    Then navigate is an Action stereotyped loop labelled "navigate [for wp in waypoints]"
    And it contains awaitArrival then advance with a succession between them

  Scenario: control nodes, successions and flows come from the subject's lists
    Given the derived Mission action
    Then start is a Fork and end a Join
    And the declared successions start → takeoff, navigate → land and land → end are edges
    And the flow takeoff.alt → navigate.alt is a Flow edge between the two steps
    And edge ids are deterministic

  Scenario: initial and final bracket the flow only when successions exist
    Given the derived Mission action
    Then the first node is the Initial node feeding only the fork
    And only the join reaches the Final node
    Given an Action diagram of Beh::Unordered, which declares no succession
    When the diagram is derived
    Then two steps are drawn with no Initial, no Final and no edges

  Scenario: include and exclude filter steps and control nodes and flag unknown entries
    Given include [takeoff, Beh::Mission::land, ghost]
    When the diagram is derived
    Then only takeoff and land remain and no control node
    And exactly one W417 names ghost
    Given exclude [checkWeather]
    When the diagram is derived
    Then the decision, its branches, its merge and every edge touching them are gone with no W417

  Scenario: a PartDef subject is W418 and draws nothing
    Given an Action diagram whose subject is the PartDef Beh::Airframe
    When the model is validated
    Then the graph has no nodes and exactly one W418 names the PartDef type

  Scenario: the writers accept the derived graph
    Given the derived Mission action
    Then render_mermaid yields a flowchart TD with {{ }} diamonds, [[ ]] bars, a loop subgraph and [then] labels
    And render_svg lays the graph out via the embedded ELK with decision, fork, join, merge, initial, final and succession classes
    And render_plantuml returns None for the Action kind
```
