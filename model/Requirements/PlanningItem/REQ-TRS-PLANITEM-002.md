---
type: Requirement
id: REQ-TRS-PLANITEM-002
name: "A PlanningItem has at most one parent, with a computed children reverse index"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PLANITEM-000]
breakdownAdr: Decisions::PlanningItemADR
tags:
  - planning
  - traceability
---

A `PlanningItem` shall accept an optional `parent: <PI-id-or-qualified-name>` field naming at most
one other `PlanningItem`. Syscribe shall compute a reverse index `children` on the parent, listing
every `PlanningItem` naming it, exactly as `Requirement.derivedFrom`/`derivedChildren` already work
(`CLAUDE.md` §11.11). A `PlanningItem` with no `parent:` is a **top-level** item.

## Rationale

Strict single-parent — a tree, not a DAG — was a deliberate choice over the more general structure
Graph-of-Thoughts-style research would suggest, made explicit in `ADR-SYS-PLANITEM-001`: it mirrors
an already-proven precedent (`Requirement`'s own breakdown mechanism) rather than introducing new
multi-parent reverse-index and leaf-detection complexity for a convergent-evidence case that's
already expressible by two different `PlanningItem`s independently citing the same evidence value,
without needing a shared graph node for it.

## Scope

- A `parent:` cycle (a `PlanningItem` transitively naming itself as an ancestor) is a validation
  error, following the same cycle-detection posture already established for other hierarchical
  references in this codebase (`CLAUDE.md`'s "Qualified name resolution handles circular references
  gracefully").
- A `parent:` naming something that isn't itself a `PlanningItem`, or that doesn't resolve at all,
  is a validation error.
- "Leaf" (for `REQ-TRS-PLANITEM-006`'s evidence rule) means a `PlanningItem` with an empty computed
  `children` index — not a separately-authored flag.
- Whether a top-level `PlanningItem` must itself resolve to something (a `Requirement` link) is
  `REQ-TRS-PLANITEM-003`, not this requirement.
