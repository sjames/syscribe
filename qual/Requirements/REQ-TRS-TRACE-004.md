---
id: REQ-TRS-TRACE-004
type: Requirement
name: Tool shall emit W300 for an unassigned approved leaf Requirement
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** emit warning `W300` for any leaf `Requirement` (no `derivedChildren`) that has `status: approved` or `status: implemented` and is not referenced by any element's `satisfies:` field.

**Source:** §12.3; §11.12 `W300`

**Acceptance criteria:** An approved leaf `Requirement` with no `satisfies:` reference anywhere in the model produces `W300`. Adding `satisfies: [REQ-*]` to an architecture element removes the warning.
