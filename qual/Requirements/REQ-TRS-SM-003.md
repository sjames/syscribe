---
id: REQ-TRS-SM-003
type: Requirement
name: Tool shall enforce flat state-machine completeness (W070–W074) on single-region StateDefs
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** enforce the SysMLv2 state-machine completeness checks of §22.1 on every
`StateDef` / `State` element that declares a non-empty `subStates:` list and is a **single
region** — i.e. **not** `isParallel: true` and with **no composite substate** (no substate
carrying `typedBy:`, an inline `subStates:`, or `isParallel:`). Parallel and composite state
machines are out of scope for this flat check (covered by later region/hierarchy-aware
phases) and **shall not** raise W070–W074.

Edges are taken from the canonical transition extractor of [[REQ-TRS-SM-001]] (both nested
and top-level placements). Over the directed graph `(source-substate → target-substate)`:

| Code | Condition |
|---|---|
| `W070` | **Dead state** — a substate with no incoming transition (in-degree 0) that is not `isInitial: true`. |
| `W071` | **Trap state** — a substate with no outgoing transition (out-degree 0) that is not `isFinal: true`. |
| `W072` | **Non-determinism** — two or more transitions from the same source with the same `accept` payload where **none** carries a `guard`. |
| `W073` | **Missing initial** — a single-region `StateDef` with substates has no `isInitial: true` substate. |
| `W074` | **Multiple initial** — more than one substate is `isInitial: true`. |

All five codes **shall** be **draft-suppressed** (not emitted when the element is
`status: draft`) and **gateable** with `--deny W07x`, consistent with the other
state-machine warnings.

The tool's demonstration models **shall** be free of W070–W074: `model`
(`FlightStates`) and `model_mg` (`ChargingSessionStates`, post-migration) are both
single-region, fully connected machines with exactly one initial state.

**Source:** §22.1 (State Machine Completeness), GH #68. Flat single-region check built on the
canonical edge model of [[REQ-TRS-SM-001]]; reachability and region/hierarchy refinements
follow in later phases ([[REQ-TRS-SM-002]] precedes this as the schema foundation).

**Acceptance criteria:**

- A single-region machine with a substate that has no incoming transition (not initial)
  raises `W070`; a fully connected machine does not.
- A substate with no outgoing transition that is not `isFinal` raises `W071`; a final state
  with no outgoing transition does not.
- Two unguarded transitions from one source with the same `accept` payload raise `W072`;
  the same pair with guards does not.
- A machine with substates but no `isInitial` raises `W073`; with two `isInitial` raises
  `W074`.
- All five are suppressed for `status: draft` and gate non-zero under `--deny W07x`.
- A `isParallel: true` or composite (`typedBy`/nested-`subStates`) machine raises none of
  W070–W074.
