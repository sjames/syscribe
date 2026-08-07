---
type: Requirement
id: REQ-TRS-PLANITEM-006
name: "A leaf PlanningItem marked done must have at least one non-waived evidence entry"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PLANITEM-000]
breakdownAdr: Decisions::PlanningItemADR
tags:
  - planning
  - validation
---

A leaf `PlanningItem` (`REQ-TRS-PLANITEM-002`: empty computed `children`) with `status: done` shall
have at least one `evidence:` entry that resolves successfully (a `ref:` that resolves, or a `path:`
that exists locally or is an accepted remote URI) — an entry excused by its own `rationale:`
(`REQ-TRS-PLANITEM-005`) does not count toward satisfying this rule, since it has been explicitly
told not to be checked. A leaf `PlanningItem` in any other status (`todo`/`in_progress`/`blocked`)
raises nothing regardless of its `evidence:` content.

## Rationale

Evidence is proof of completion, not a prerequisite for starting — so the rule is graded by status,
not a blanket "every leaf needs evidence" check. This is a **harder** severity than the analogous
`Requirement` rule (`W300`: a leaf `Requirement` at `approved`/`implemented` with no satisfying
element is a warning, because it may simply not have reached that point in its lifecycle yet):
claiming `status: done` with zero resolvable evidence is an outright correctness defect, not a
time-bound gap, so this rule is an error.

## Scope

- Only leaf `PlanningItem`s are checked; a non-leaf (has `children`) `PlanningItem`'s own `status`
  and `evidence:` are not constrained by this rule — its completion is a function of its children,
  not its own evidence list. (Whether/how a parent's status should be validated against its
  children's statuses is not addressed by this requirement and is left as a follow-on question if a
  concrete need arises.)
- A waived-only evidence list (every entry carries `rationale:`) on a `status: done` leaf still
  raises the error — a rationale excuses one entry's *check*, it does not manufacture a passing
  entry to satisfy the "at least one" count.
- Exact error code is assigned at implementation time, in a range distinct from the existing
  `Requirement`/`TestCase` traceability codes (`E310`–`E315`, `W300`–`W304`).
