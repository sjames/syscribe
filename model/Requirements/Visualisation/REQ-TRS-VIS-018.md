---
type: Requirement
id: REQ-TRS-VIS-018
name: "The StateMachine generator derives states, pseudostates and labelled transitions from a StateDef or State subject"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-VIS-015]
breakdownAdr: Decisions::VisualisationADR
tags:
  - diagram
  - visualisation
  - statemachine
---

For a derived `diagramKind: StateMachine` whose subject is a `StateDef`, `State` or
`ExhibitState` (a `State`/`ExhibitState` subject reads the `StateDef` it is typed by), the
generator shall produce:

- one `State` node per `subStates:` entry (spec §8.8.2), labelled by its `name`, with compartment
  lines `entry / <action>`, `do / <action>` and `exit / <action>` for `entryAction`/`doAction`/
  `exitAction` (string form: the last `::` segment; map form: its `name` or `typedBy`); a substate
  whose `typedBy:` resolves to a `StateDef` with its own `subStates:` becomes a container holding
  that machine's states, one level deep;
- one `Initial` pseudostate node per region with a `Transition` edge to each `isInitial: true`
  state, and one `Final` node that every `isFinal: true` state transitions to;
- one `Transition` edge per transition in either placement (nested under a substate, or
  top-level with `source:`; the deprecated `from`/`to`/`trigger` aliases are read like the
  canonical keys, exactly as the `W07x` extractor does), labelled `<accept> [<guard>] / <effect>`
  with the absent parts omitted — `accept` is the payload's last segment (or `after`/`when`/`at`
  for time/change triggers), `effect` is the effect's `name` or last `typedBy` segment;
- an edge only when both endpoint states are nodes of the diagram; a transition with a missing
  endpoint produces no edge and is left to `W929`.

Shape ids are `derived_shape_id("<subject>::<stateName>")`; the pseudostates are
`<subjectId>-initial` and `<subjectId>-final`. Layout hints: layered, top-to-bottom.

## Rationale

The HSM work (`REQ-TRS-SM-*`) made state machines first-class; the demo `FlightStates` has six
states and eleven transitions that no hand-listed manifest keeps in step with the model.

## Scope

- Parallel regions (`isParallel: true`) are drawn as sibling containers when the substates are
  themselves typed machines; orthogonal-region dividers are not drawn.
- History and choice pseudostates are not generated (no model field declares them).
