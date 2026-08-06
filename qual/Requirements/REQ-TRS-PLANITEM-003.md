---
id: REQ-TRS-PLANITEM-003
type: Requirement
name: A top-level PlanningItem shall declare which Requirements it exists to achieve via achieves, independent of satisfies
status: draft
reqDomain: software
verificationMethod: test
---

A `PlanningItem` **shall** accept an `achieves: <Requirement id-or-qname | list>` field naming one
or more native `Requirement`s. A **top-level** `PlanningItem` (no `parent:`) **shall** set at least
one `achieves:` entry; a non-top-level `PlanningItem` is not required to. An `achieves:` target
that does not resolve, or that resolves to something other than a native `Requirement`, **shall**
be reported as a validation error. `achieves:` **shall not** participate in the existing
`satisfies:`-scoped traceability rules — it shall never suppress `W300` (leaf-requirement
coverage) and shall never trigger `E312` (no-parent-assignment) on its target.

**Source:** `REQ-TRS-PLANITEM-003` (product model), `ADR-SYS-PLANITEM-001` Decision 2.

**Acceptance criteria:** a top-level `PlanningItem` with a resolving `achieves:` (to a native
`Requirement`) validates cleanly; a top-level item with no `achieves:` is rejected; a dangling or
wrong-type `achieves:` target is rejected; a `Requirement` named only via `achieves:` (never
`satisfies:`) still raises `W300` when otherwise eligible, and a parent `Requirement` named only
via `achieves:` never raises `E312`.
