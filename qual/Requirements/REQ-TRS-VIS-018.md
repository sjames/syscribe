---
id: REQ-TRS-VIS-018
type: Requirement
name: The StateMachine generator derives states, pseudostates and labelled transitions from a StateDef or State subject
status: verified
reqDomain: software
verificationMethod: test
---

For a derived `diagramKind: StateMachine` whose subject is a `StateDef`, `State` or
`ExhibitState` (a `State`/`ExhibitState` subject reads the `StateDef` it is typed by), the
generator **shall** produce:

- one `State` node per `subStates:` entry (spec §8.8.2), labelled by its `name`, with
  compartment lines `entry / <action>`, `do / <action>` and `exit / <action>` for
  `entryAction`/`doAction`/`exitAction` (string form: the last `::` segment; map form: its
  `name` or `typedBy`); a substate whose `typedBy:` resolves to a `StateDef` with its own
  `subStates:` **shall** become a container holding that machine's states, one level deep;
- one `Initial` pseudostate node per region with a `Transition` edge to each
  `isInitial: true` state, and one `Final` node that every `isFinal: true` state transitions
  to;
- one `Transition` edge per transition in either placement (nested under a substate, or
  top-level with `source:`; the deprecated `from`/`to`/`trigger` aliases read like the
  canonical keys, exactly as the `W07x` extractor does), labelled `<accept> [<guard>] / <effect>`
  with the absent parts omitted — `accept` is the payload's last segment (or `after`/`when`/`at`
  for time/change triggers), `effect` is the effect's `name` or last `typedBy` segment;
- an edge only when both endpoint states are nodes of the diagram; a transition with a missing
  endpoint produces no edge and is left to `W929`.

Shape ids **shall** be `derived_shape_id("<subject>::<stateName>")`; the pseudostates
`<subjectId>-initial` and `<subjectId>-final`. Layout hints: layered, top-to-bottom. A subject
of any other type **shall** raise `W418` and produce no nodes; `include:`/`exclude:` **shall**
apply to the top-level substates by name or qualified name.

**Source:** `REQ-TRS-VIS-018` (product model).

**Acceptance criteria:** through the real walker and validator, on a fixture `StateDef`
(`Beh::Flight`: `disarmed` initial, `armed` with an `entryAction`, `flying` typed by a
sub-machine with a map-form `doAction`, `landed` final with an `exitAction`; nested transitions
in both `accept` spellings, one top-level transition in the deprecated aliases, one transition
to a missing state), (a) the four top-level states appear in declaration order and the golden
IR matches `tests/vis_snapshots/derived/flight_sm.json`; (b) compartments read
`entry / Takeoff`, `do / navigate`, `exit / Land` and a state without actions has none; (c) the
transition labels are `Command [armed == false]`, `Command [ready] / startTakeoff`,
`[altitude <= 0.1] / Land` and, for the aliased top-level transition, `Command`, every edge is
a `Transition`, and a named transition is referenced by `<subject>::<name>`; (d) an `Initial`
node has an edge to `disarmed` and `landed` has one to the `Final` node; (e) `flying` is a
container holding `hold`/`track` with its own `-initial`/`-final` pseudostates and region
edges; (f) the transition to the missing state draws no edge and the diagram raises no
`W417`/`W418`; (g) `include: [disarmed, Beh::Flight::armed, limbo]` keeps two states and two
edges with one `W417` naming `limbo`, and `exclude: [flying]` removes the container, its
region and every edge touching it; (h) a `PartDef` subject yields no nodes and one `W418`
naming the type; (i) `render_mermaid` returns a `stateDiagram-v2` with the composite state
nested, `render_svg` lays the graph out, and `render_plantuml` emits `[*]` for the
pseudostates.
