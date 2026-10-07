---
id: TC-TRS-VIS-021
type: TestCase
testLevel: L2
status: active
name: "Verify the Sequence generator derives participants, messages, fragments and the activation from an ActionDef's send/accept actions in succession order, pins everything with horizontal message waypoints, filters participants, reports a wrong subject as W418, raises no W080, and is drawn by every writer."
verifies:
  - REQ-TRS-VIS-021
sourceFile: repo:crates/syscribe-model/tests/vis_derive_sequence.rs
testFunctions:
  - derived_sequence_of_an_actiondef_matches_its_golden_ir
  - participants_appear_subject_first_in_first_appearance_order_then_actors
  - messages_follow_the_successions_and_descend_into_branches_and_bodies
  - fragments_span_the_messages_they_contain
  - the_activation_sits_on_the_subjects_lifeline_spanning_its_messages
  - every_node_is_pinned_and_every_message_runs_horizontally_between_stems
  - a_partdef_subject_is_w418_and_draws_nothing
  - filters_keep_or_drop_participants_by_qualified_or_short_name
  - a_derived_sequence_diagram_raises_no_w080_while_a_manifest_still_does
  - every_writer_accepts_the_derived_sequence
tags:
  - diagram
  - visualisation
  - sequence
---

Hosted integration tests in `crates/syscribe-model/tests/vis_derive_sequence.rs`; run with
`cargo test -p syscribe-model --test vis_derive_sequence`. Each test writes a small model to a
temp directory — a flight controller that performs `Mission` and owns the `gpsIn` port, a
propulsion part, a ground-station actor, and the `Mission` action with two perform steps, a
send `to` a part, an accept `via` a port, an `IfAction` with a send in each branch (the else
to an unresolvable chain), a `LoopAction` with a body send and successions that reorder the
declaration — runs the real walker and validator on it and derives the Sequence diagram. The
golden IR is `tests/vis_snapshots/derived/mission_seq.json`.

```gherkin
Feature: the Sequence generator (TC-TRS-VIS-021)

  Scenario: golden IR
    Given a derived Sequence diagram of the fixture Mission action
    When the IR is built
    Then it matches the committed snapshot with no W417, W418 or E405

  Scenario: participants
    Then the lifelines are Mission, FlightController (owner of gpsIn), Propulsion, a dashed relay.link, then the GroundStation actor

  Scenario: message order
    Then the messages are awaitFix, setThrottle, steer, abort, notifyRelay — the succession order, descending into the loop body and both if branches
    And each is labelled name(Payload) and rows descend

  Scenario: fragments and activation
    Then the loop fragment encloses steer, the alt fragment encloses abort and notifyRelay, and they do not overlap
    And the activation is a child of the subject's lifeline spanning the first to the last row

  Scenario: self-placement
    Then every lifeline is pinned at its column with the header size, every message has two horizontal waypoints at the stems
    And is_fully_pinned holds and the pinned layout places every node and edge without ELK

  Scenario: wrong subject
    Given a Sequence diagram whose subject is a PartDef
    Then exactly one W418 names PartDef and the graph is empty

  Scenario: filters
    Given include naming Propulsion by qualified name, GroundStation by short name and Ghost
    Then only those participants and the subject remain, with one W417 for Ghost
    Given exclude naming FlightController
    Then that lifeline and its message are gone

  Scenario: W080
    Given the derived diagram with status approved
    Then no W080 is raised, while a manifest diagram of the same subject listing one edge still raises it

  Scenario: writers
    Then render_svg succeeds through the pinned path with dashed stems, a stick figure, the activation, the fragment tab and the message along its row
    And render_mermaid and render_plantuml both return a sequence diagram
```
