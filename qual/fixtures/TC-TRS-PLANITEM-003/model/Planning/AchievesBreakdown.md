---
type: PlanningItem
id: PI-P3-BREAKDOWN-001
name: "Achieves the parent/child breakdown pair"
status: in_progress
achieves: [REQ-P3-PARENT-001, REQ-P3-CHILD-001]
---

Deliberately achieves both `REQ-P3-PARENT` (has `derivedChildren`) and `REQ-P3-CHILD` (a leaf),
neither ever named in a `satisfies:` list — proves `achieves:` never triggers `E312` on the
parent and never suppresses `W300` on the leaf.
