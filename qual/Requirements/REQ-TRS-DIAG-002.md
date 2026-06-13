---
id: REQ-TRS-DIAG-002
type: Requirement
name: Tool shall enforce Sequence diagram send/receive completeness (W080)
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** enforce the `Sequence` diagram send/receive completeness rule
defined in §22.4 (extending §8.16.8.3) of the Syscribe format specification.

For every `type: Diagram` element `D` whose `diagramKind` is `Sequence` and whose
`subject:` resolves to an `ActionDef` `A`, the tool **shall** emit **`W080`** for
every `SendAction` and every `AcceptAction` reachable through `A`'s sub-action tree
(`subActions:` / `steps:`, descending recursively into `then:` / `else:` branches of
`IfAction`s and nested `subActions:`) whose message event is **not referenced** by any
entry in `D`'s `edges:` list — i.e. the sequence diagram is missing an edge for a known
message event.

A message action is considered **referenced** when some `edges:` entry's `ref:` value
equals the action's qualified name (`<A>::<actionName>`) or its bare `actionName`.

### Scope and gating

- `W080` **shall** apply only to `Diagram` elements with `diagramKind: Sequence` and a
  `subject:` that resolves to an `ActionDef`; a `subject:` resolving to any other type,
  or not resolving, raises no `W080` (the unresolved case is already `W401`,
  [[REQ-TRS-DIAG-001]]).
- `W080` **shall** be **draft-suppressed**: a `Sequence` diagram with `status: draft`
  emits no `W080`.
- `W080` is a warning and **shall** be gateable with `--deny W080`, promoting it to a
  gate failure (non-zero exit).

**Source:** §22.4 (Sequence Diagram Send/Receive Completeness), GH #70. The completeness
rule was previously documented as advisory in §8.16.8.3 but never emitted; this makes it
normative.

**Acceptance criteria:**

- A `Sequence` diagram whose subject `ActionDef` has a `SendAction` with no covering
  `edges:` entry produces a `W080` finding naming that action.
- A `Sequence` diagram in which every `SendAction`/`AcceptAction` of the subject is
  covered by an `edges:` entry produces **no** `W080`.
- A `SendAction` nested inside an `IfAction` `then:` / `else:` branch is reached by the
  recursion (covered actions clear, uncovered actions raise `W080`).
- The same defective `Sequence` diagram with `status: draft` produces **no** `W080`.
- Running with `--deny W080` over a model containing an uncovered message action exits
  non-zero.
- The demo model (`model/`) — whose `MissionExecutionSeq` covers `abortMission` via the
  `e-abort` edge — produces **no** `W080`.
