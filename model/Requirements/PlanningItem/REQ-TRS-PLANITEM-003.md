---
type: Requirement
id: REQ-TRS-PLANITEM-003
name: "A top-level PlanningItem declares which Requirements it exists to achieve via achieves:"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PLANITEM-000]
breakdownAdr: Decisions::PlanningItemADR
tags:
  - planning
  - traceability
---

A `PlanningItem` shall accept an `achieves: <Requirement id-or-qname | list>` field naming one or
more native `Requirement`s. A **top-level** `PlanningItem` (`REQ-TRS-PLANITEM-002`, no `parent:`)
**shall** set at least one `achieves:` entry — its reason for existing is to achieve those
requirements' goals. A non-top-level `PlanningItem` may also set `achieves:` but is not required to;
its purpose is inherited in spirit from its ancestry, not re-declared at every level.

## Rationale

`achieves:` is a new, dedicated field rather than a reuse of the existing `satisfies:` field
architecture elements already use to target a `Requirement`. `satisfies:` carries real, specific
validation machinery (`E312`–`E315`'s domain-matching and leaf-satisfaction rules) scoped to
architecture semantics; overloading it for planning-item purposes risks either silently inheriting
rules that don't apply to planning work or requiring type-based carve-outs sprinkled through that
machinery. A distinct field name keeps both concerns clean, at the cost of one new name.

## Scope

- A top-level `PlanningItem` with an empty or absent `achieves:` is a validation error.
- An `achieves:` target that doesn't resolve to a real `Requirement` is a dangling-reference
  finding, the same class already raised for any other unresolved cross-reference — checked
  empirically at implementation time (per the lesson from the SysMLv2 `satisfy`/`verify` work,
  where an assumed-symmetric dangling-reference diagnostic turned out not to exist for one of two
  related fields) rather than assumed.
- `achieves:` does not participate in `Requirement`'s existing leaf-satisfaction (`W300`) or
  no-parent-assignment (`E312`) rules — those remain scoped to architecture `satisfies:` only.
