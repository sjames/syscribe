---
id: REQ-TRS-TRACE-005
type: Requirement
name: "Tool shall emit E312 when a parent Requirement is used in satisfies:"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** emit error `E312` when a `Requirement` that has `derivedChildren` (i.e. is a parent in the breakdown hierarchy) appears as a target in any element's `satisfies:` list.

**Source:** §12.4; §11.12 `E312`

**Acceptance criteria:** An architecture element with `satisfies: [REQ-PARENT-001]` where `REQ-PARENT-001` has derived children produces `E312`.
