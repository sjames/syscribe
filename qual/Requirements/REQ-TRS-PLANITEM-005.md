---
id: REQ-TRS-PLANITEM-005
type: Requirement
name: PlanningItem evidence shall be a list of ref/path entries, each independently waivable by its own rationale
status: draft
reqDomain: software
verificationMethod: test
---

A `PlanningItem` **shall** accept an `evidence:` field — a list of duck-typed entries, each
carrying `ref: <id-or-qname>` (any resolvable element, unrestricted by kind) or
`path: <local-path-or-URI>` (resolved exactly like `implementedBy:`: a local path is checked to
exist, a remote URI is accepted as external without a local check). Either form **may**
additionally carry `rationale: <string>`, which waives *that entry's own* existence/resolution
check. An unresolved `ref:` or a non-existent local `path:` **shall** be a validation error
**unless** the same entry also carries a non-empty `rationale:`, in which case the check **shall**
be skipped for that entry only — every other entry in the same list **shall** still be checked
normally.

**Source:** `REQ-TRS-PLANITEM-005` (product model), `ADR-SYS-PLANITEM-001` Decision 3.

**Acceptance criteria:** a resolving `ref:` and an existing local `path:` each validate cleanly; a
dangling `ref:` and a missing local `path:` are each rejected; the same dangling `ref:`/missing
`path:` with a `rationale:` is not rejected; a remote-URI `path:` is accepted with no local check;
a list mixing one `rationale:`-waived broken entry with one genuinely broken, un-waived entry
flags only the un-waived one.
