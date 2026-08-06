---
type: Requirement
id: REQ-RTH-002
name: "Return-to-Home events logged for post-flight review"
status: approved
reqDomain: software
reqClass: derived
derivedFrom: [REQ-RTH-001]
breakdownAdr: Decisions::ADR-RTH-001
tags:
  - rth
  - logging
---

The flight controller shall record the trigger reason, GPS position, and
remaining battery capacity at the moment a return-to-home event is initiated,
and shall persist this record for retrieval during post-flight review.

## Rationale

Derived from `REQ-RTH-001` per `ADR-RTH-001`. Achieved by
`Planning::PI-RTH-001` (`achieves:`, id-form list entry) — deliberately **not**
`satisfies:`, so this leaf requirement still raises `W300` (no satisfying
architecture element) even though a `PlanningItem` is actively working toward
it. See the README's "Expected / documented warnings" section: this is the
intended, designed-in separation between `achieves:` and `satisfies:`
(`ADR-SYS-PLANITEM-001` Decision 2), not an oversight.
