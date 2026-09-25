---
id: REQ-TRS-TRACE-011
type: Requirement
name: Tool shall treat a SafetyGoal or CybersecurityGoal derivation as upstream traceability for the orphan warning W005
status: draft
reqDomain: software
verificationMethod: test
---

Warning `W005` flags a native `Requirement` that is connected to no
requirement hierarchy — it has no upstream link and no `derivedChildren`. The
tool **shall** count each of the following as an upstream link for `W005`, so
a requirement carrying any one of them is **not** reported as an orphan:

- a non-empty `derivedFrom:`;
- a non-empty `derivedFromSafetyGoal:` (the `SafetyGoal` that generated it);
- a non-empty `derivedFromCybersecurityGoal:`, including its legacy alias
  `derivedFromSecurityGoal:` (the `CybersecurityGoal` that generated it).

A goal-derived requirement is traced upstream to the goal exactly as a
`derivedFrom:` requirement is traced to its parent requirement; whether the
goal reference resolves is checked separately (`E832`/`E831`), not by `W005`.
A requirement with none of these links and no derived children **shall** still
raise `W005`.

**Source:** GH issue #151.

**Acceptance criteria:** a requirement whose only upstream link is
`derivedFromSafetyGoal:`, `derivedFromCybersecurityGoal:` or
`derivedFromSecurityGoal:` raises no `W005`; a requirement with no upstream link
and no derived children still raises exactly one `W005`.
