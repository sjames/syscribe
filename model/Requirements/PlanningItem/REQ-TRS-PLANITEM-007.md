---
type: Requirement
id: REQ-TRS-PLANITEM-007
name: "A PlanningItem may declare blockedBy: naming what it's waiting on, validated for dangling references, cycles, and status/field consistency"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PLANITEM-000]
breakdownAdr: Decisions::PlanningItemADR
tags:
  - planning
  - traceability
---

A `PlanningItem` shall accept an optional `blockedBy: <ref-or-list>` field naming one or more
elements it is waiting on before it can proceed — most commonly another `PlanningItem`, but not
restricted to that kind (an undecided `ADR`, an unmet dependency elsewhere in the model are equally
legitimate blockers). Each entry shall resolve to a real model element, following the same
permissive, unrestricted-by-kind resolution `PlanningItem.evidence.ref:` already established
(`REQ-TRS-PLANITEM-005`), not a fixed allowed-kind list.

## Rationale

`REQ-TRS-PLANITEM-000` already named "dependency" alongside breakdown and completion evidence as
something this feature exists to make resolvable in-graph, but no field ever implemented it —
`status: blocked` could be declared with no way to say *what* it's blocked on. This closes that gap
the same way the rest of `PlanningItem` closes it: a plain cross-reference field, checked by the
same validator, visible in `refs`/`show`, rather than a free-text note.

## Scope

- A `blockedBy:` entry that does not resolve to any model element is a validation error, mirroring
  `achieves:`'s dangling check (`REQ-TRS-PLANITEM-003`).
- A `blockedBy:` cycle (transitively blocked on itself, directly or through other `PlanningItem`s)
  is a validation error, following the same cycle-detection posture as `parent:`
  (`REQ-TRS-PLANITEM-002`) — reported gracefully, never a panic or infinite loop.
- A non-empty `blockedBy:` on a `PlanningItem` whose `status` is **not** `blocked` is a warning
  (likely stale — the blocker was resolved and `status:` was never updated to reflect it), not an
  error — the field and the status label can drift independently without corrupting the model.
- The converse — `status: blocked` with an **empty** `blockedBy:` — raises nothing. "Blocked, reason
  not yet captured" is a legitimate transient state, unlike claiming `done` with no evidence
  (`REQ-TRS-PLANITEM-006`): being blocked needs no proof, only completion does.
- Whether a blocker resolving to something itself `done`/closed should auto-clear `blockedBy:` or
  flag it as stale is not addressed here — `blockedBy:` is a plain, author-maintained cross-reference
  field, not a computed one, exactly like `parent:`.
