---
id: REQ-TRS-SM-001
type: Requirement
name: Tool shall recognise one canonical SysMLv2 state-transition schema (source/target/accept/guard/effect)
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** recognise a **single canonical** transition schema for `StateDef` /
`State` elements, faithful to SysMLv2 (`temp/sysml2_spec.pdf` §7.18.3, transition usage
`transition first <source> accept <trigger> if <guard> do <effect> then <target>`).

A transition entry **shall** carry the following fields:

| Field | Form | Meaning |
|---|---|---|
| `source` | local name or qualified name | The source state. **Optional** when the transition is lexically nested inside a `subStates:` entry (the enclosing substate is the implicit source); **required** for a top-level `transitions:` entry. |
| `target` | local name or qualified name | **Required.** The target state. |
| `accept` | string **or** `{payload: <qn>, via: <featureChain>}` | Optional trigger (AcceptAction). The string shorthand `accept: X` is equivalent to `accept: {payload: X}`. |
| `guard` | string (opaque Boolean expression) | Optional guard condition. |
| `effect` | string **or** `{name, typedBy}` | Optional effect action; string shorthand is a qualified name. |

The tool **shall** accept transitions authored in **either placement** and treat them as
the same edge model:

1. **nested** — under a `subStates:` entry's own `transitions:` list (source implicit);
2. **top-level** — under the `StateDef`'s `transitions:` list (`source:` explicit).

A single **transition extractor** **shall** yield the directed edge set
`(source-state → target-state)` from both placements, so every downstream check (dead/trap,
non-determinism, initial/final, reachability) consumes one consistent edge model
regardless of authoring placement.

`isInitial: true` and `isFinal: true` on a `subStates:` entry are the canonical shorthands
for the SysMLv2 `entry; then <state>` (initial) and `then done` (final) successions.

**Source:** SysMLv2 §7.18 (State Machine Elements); plan to unify the format's two
divergent transition spellings. Pairs with the deprecation of the legacy `from`/`to`/
`trigger` keys in [[REQ-TRS-SM-002]].

**Acceptance criteria:**

- A `StateDef` with nested per-substate `transitions:` using `target`/`accept`/`guard`
  validates clean and its edges are recognised.
- A `StateDef` with a top-level `transitions:` list using `source`/`target` validates clean
  and its edges are recognised identically to the nested form.
- `accept: Items::Foo` (string) and `accept: {payload: Items::Foo}` (map) are treated
  equivalently.
- The edge set extracted is independent of whether a transition is authored nested or
  top-level.
