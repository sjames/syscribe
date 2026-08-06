---
type: PlanningItem
id: PI-RTH-IMPL-SW-002
name: "Implement home-position flight-path replanner"
status: todo
itemType: task
parent: PI-RTH-IMPL-001
tags:
  - rth
---

Implement the replanning algorithm that computes the return path to the
recorded home position, avoiding active no-fly zones. Not yet started —
`status: todo`. A leaf `PlanningItem` with no `evidence:` at all, which is
perfectly fine: `REQ-TRS-PLANITEM-006`'s leaf-evidence rule only applies at
`status: done` (evidence is proof of completion, not a prerequisite for
starting).
