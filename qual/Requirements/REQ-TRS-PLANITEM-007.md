---
id: REQ-TRS-PLANITEM-007
type: Requirement
name: A PlanningItem may declare blockedBy naming what it is waiting on, validated permissively for dangling references, cycles, and status consistency
status: draft
reqDomain: software
verificationMethod: test
---

A `PlanningItem` **shall** accept an optional `blockedBy: <ref-or-list>` field naming one or more
elements it is waiting on before it can proceed. Each entry **shall** be resolved permissively —
any model element, not restricted to `PlanningItem` — mirroring `evidence.ref:`'s unrestricted-by-
kind resolution. A `blockedBy:` entry that does not resolve **shall** be reported as a validation
error. A `blockedBy:` chain that cycles back to itself, directly or through other `PlanningItem`s,
**shall** be reported as a validation error, gracefully — never a panic or an infinite loop. A
non-empty `blockedBy:` on a `PlanningItem` whose `status` is not `blocked` **shall** be reported as
a warning (likely stale); the converse — `status: blocked` with an empty or absent `blockedBy:` —
**shall** raise nothing.

**Source:** `REQ-TRS-PLANITEM-007` (product model), `ADR-SYS-PLANITEM-001` addendum.

**Acceptance criteria:** a `blockedBy:` naming a real, resolvable element (a `PlanningItem` or any
other kind) validates cleanly with no cross-tier hierarchy error; a dangling `blockedBy:` is
rejected; a 2-node `blockedBy:` cycle is rejected, with no crash; a non-empty `blockedBy:` on a
`status` other than `blocked` is warned; `status: blocked` with no `blockedBy:` raises nothing at
all.
