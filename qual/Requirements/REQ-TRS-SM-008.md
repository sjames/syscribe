---
id: REQ-TRS-SM-008
type: Requirement
name: Tool shall resolve state-machine behavior references (W079) and accept decision transitions
status: draft
reqDomain: software
verificationMethod: test
---

This completes the SysMLv2 state-machine support with behavior-reference integrity and an
explicit guarantee for decision (guarded-branch) transitions.

### Behavior reference resolution (`W079`)

A `StateDef` / `State` may attach behaviors via a state's `entryAction:` / `doAction:` /
`exitAction:` and a transition's `effect:` (SysMLv2 §7.18). Each names an action, either as
a qualified-name string shorthand or as a `{typedBy: <qn>}` map. The tool **shall** emit
**`W079`** for any such reference — collected recursively across the whole state hierarchy —
that does **not** resolve to a model element.

- `W079` **shall** be **draft-suppressed** and **gateable** with `--deny W079`. Each
  distinct unresolved reference is reported once per element.
- `accept.payload` is **not** subject to `W079`: payloads frequently name informal event
  labels rather than model elements, so payload resolution is out of scope here.

### Decision transitions are legitimate

Per SysMLv2 §7.18 / §8.4.13.3, **decision transitions** — two or more transitions from the
same source distinguished by **guards** — are a valid branching construct. The tool
**shall not** raise the non-determinism warning `W072` for same-source transitions when they
carry guards; `W072` is reserved for same-source, same-payload transitions where **none** is
guarded (already specified by [[REQ-TRS-SM-003]]). This requirement records that guarantee
and is covered by a regression check.

**Source:** SysMLv2 §7.18 (entry/do/exit and transition effect actions) and §8.4.13.3
(decision transition usages). Final phase of the state-machine work
([[REQ-TRS-SM-006]], [[REQ-TRS-SM-007]]).

**Acceptance criteria:**

- A transition `effect:` (string or `{typedBy}`) naming a non-existent action raises `W079`;
  one that resolves does not.
- A state `entryAction:`/`doAction:`/`exitAction:` naming a non-existent action raises
  `W079`; one that resolves does not.
- An `accept.payload` that does not resolve raises **no** `W079`.
- Two guarded transitions from the same source with the same payload raise **no** `W072`.
- `W079` is suppressed for `status: draft` and gates non-zero under `--deny W079`.
- The shipped models are `W079`-clean.
