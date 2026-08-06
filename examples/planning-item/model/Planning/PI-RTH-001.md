---
type: PlanningItem
id: PI-RTH-001
name: "Implement automatic Return-to-Home on critical battery"
status: in_progress
itemType: feature
achieves: [REQ-RTH-001, REQ-RTH-002]
tags:
  - rth
---

Top-level breakdown of the work to add automatic return-to-home behavior when
battery capacity drops below the critical threshold. No `parent:` — this is a
**top-level** `PlanningItem` (`REQ-TRS-PLANITEM-002`), so it must (and does)
set `achieves:` (`REQ-TRS-PLANITEM-003`): the id-form list
`[REQ-RTH-001, REQ-RTH-002]`, targeting both the top-level stakeholder goal
and one of its derived requirements directly.

It is a **non-leaf** — `Planning::PI-RTH-DESIGN-001`,
`Planning::PI-RTH-IMPL-001`, `Planning::PI-RTH-TEST-001`,
`Planning::PI-RTH-DOCS-001`, `Planning::PI-RTH-BUGFIX-001`, and
`Planning::PI-RTH-CLOUDLOG-001` all set `parent: PI-RTH-001`, so its computed
`children` index is non-empty. `REQ-TRS-PLANITEM-006`'s leaf-evidence rule
therefore does not constrain this item at all, regardless of its own
`status`/`evidence:` (it has neither).
