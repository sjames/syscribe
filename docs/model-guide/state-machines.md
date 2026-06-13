# State Machines

`GUIDE · STATE MACHINES`

Syscribe models state-based behaviour with `StateDef` / `State` elements, faithful to
SysMLv2 §7.18. A state machine is a composite state whose `subStates:` are connected by
**transitions**; the tool validates it for completeness (dead/trap states, missing or
duplicate initials, non-determinism, illegal cross-region transitions, and unresolved
references) with the `W070`–`W079` checks. All of these are draft-suppressed and gateable
with `--deny W07x`.

## A flat state machine

A `StateDef` lists its substates under `subStates:`. Exactly one substate is the initial
state (`isInitial: true`); final states are marked `isFinal: true`. Each substate's
`transitions:` carry it to a `target:`.

```yaml
---
type: StateDef
name: FlightStates
subStates:
  - name: disarmed
    isInitial: true
    transitions:
      - target: armed
        accept: { payload: Items::ControlCommand }
        guard: "armStatus == ArmStatus::disarmed and gps.fixQuality >= 1"
  - name: armed
    transitions:
      - target: takingOff
        accept: Items::ControlCommand
        guard: "armStatus == ArmStatus::armed"
        effect: { name: startTakeoff, typedBy: Behavior::TakeoffAction }
  - name: takingOff
    entryAction: Behavior::TakeoffAction
    transitions:
      - target: flying
        guard: "altitudeM >= targetAltitudeM"
  - name: flying
    doAction: Behavior::MissionExecution
    transitions:
      - target: disarmed
        guard: "altitudeM <= 0.1"
---
```

## The canonical transition schema (§8.8.3)

A transition is defined by one consistent vocabulary, taken from the SysMLv2 textual form
`transition first <source> accept <trigger> if <guard> do <effect> then <target>`:

| Field | Meaning |
|---|---|
| `source` | The source state. **Optional** when the transition is nested under a `subStates:` entry (that substate is the implicit source); **required** for a top-level `transitions:` entry. |
| `target` | **Required.** The target state. |
| `accept` | Trigger event — a string (`accept: Items::Cmd`) or `{ payload: <qn>, via: <port> }`. |
| `guard` | Boolean guard expression (opaque string). |
| `effect` | Effect action — a qualified-name string or `{ name, typedBy }`. |

Transitions may be authored **either** nested under a substate (source implicit) **or** at
the `StateDef` top level (with an explicit `source:`); both yield the same edge model.

`isInitial: true` and `isFinal: true` are the shorthands for the SysMLv2 `entry; then …`
(initial) and `then done` (final) pseudostates.

!!! warning "Deprecated keys (`W075`)"
    The keys `from:` / `to:` / `trigger:` are **not** SysMLv2 vocabulary. They are still
    accepted as aliases (`from`≡`source`, `to`≡`target`, `trigger`≡`accept.payload`) so old
    models keep parsing, but any transition using them raises **`W075`**. Migrate to the
    canonical keys.

## Completeness checks (flat machines)

| Code | Catches |
|---|---|
| `W070` | **Dead state** — a substate with no incoming transition that is not `isInitial`. |
| `W071` | **Trap state** — a substate with no outgoing transition that is not `isFinal`. |
| `W072` | **Non-determinism** — two transitions from one source with the same `accept` payload and no guards to disambiguate. |
| `W073` | **Missing initial** — a region with substates but no `isInitial` substate. |
| `W074` | **Multiple initial** — more than one `isInitial` substate in a region. |
| `W076` | **Unresolved endpoint** — a transition `source`/`target` that names no state in the machine and resolves to no element. |
| `W079` | **Unresolved behaviour** — an `entryAction`/`doAction`/`exitAction` or transition `effect` that resolves to no element. |

A **decision** transition (two or more *guarded* transitions from the same source) is a
legitimate branch and does **not** raise `W072` — guards are what disambiguate.

## Parallel (orthogonal) regions

Set `isParallel: true` to make a state's direct substates **concurrent regions**. Each
region is itself a composite with its own `subStates:` and its own initial state; the regions
run at the same time.

```yaml
---
type: StateDef
name: Vehicle
isParallel: true
subStates:
  - name: Motion          # region 1
    subStates:
      - { name: stopped, isInitial: true, transitions: [{ target: moving, accept: Go }] }
      - { name: moving,  transitions: [{ target: stopped, accept: Stop }] }
  - name: Lights          # region 2 — runs concurrently with Motion
    subStates:
      - { name: off, isInitial: true, transitions: [{ target: on, accept: Switch }] }
      - { name: on,  transitions: [{ target: off, accept: Switch }] }
---
```

`W073`/`W074` are then checked **per region** (each region needs exactly one initial). Two
further rules apply to the parallel parent:

- **`W077`** — a transition that connects substates in **two different regions** (SysMLv2
  forbids transitions between the substates of a parallel state).
- **`W078`** — an `isParallel: true` state with **fewer than two** regions.

## Composite (hierarchical) states

A substate is **composite** when it carries `typedBy:` (a reference to another `StateDef`)
or an inline `subStates:` list. The checks recurse: each level is validated, a composite
substate is treated as a single node in its parent's graph, and inline-`subStates:` interiors
are checked as their own region (findings name the enclosing region). A `typedBy:` substate
is a node only — the referenced `StateDef` is validated as its own element.

```yaml
subStates:
  - name: Active
    isInitial: true
    subStates:                      # inline nested machine (composite-by-containment)
      - { name: warmup,  isInitial: true, transitions: [{ target: running }] }
      - { name: running, isFinal: true }
    transitions:
      - target: Done                # Active's own (parent-level) transition
  - name: Done
    isFinal: true
```

## SysMLv2 conformance

Syscribe follows SysMLv2's streamlined state-machine model: composite + parallel states,
`entry`/`do`/`exit` actions, accept/guard/effect transitions, and **decision transitions**
for branching. It deliberately omits UML's pseudostate zoo (choice / junction / fork / join /
history) — those are not part of SysMLv2.

## See also

- [Element Types](../format/elements.md) · [Frontmatter](../format/frontmatter.md#state-machine-transitions-statedefstate)
- Full schema: `syscribe spec` (§8.8) · rule reference: [Validation Rules](../validation/rules.md#state-machine-warnings-w070w079-221)
