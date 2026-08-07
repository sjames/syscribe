---
type: PlanningItem
id: PI-P7-NONPI-001
name: "Blocked on a non-PlanningItem element"
status: blocked
achieves: REQ-P7-001
blockedBy: REQ-P7-001
---

blockedBy: resolves, but to a Requirement, not a PlanningItem -- permissive resolution means this
must still validate cleanly (no "wrong kind" check exists for blockedBy:, unlike parent:).
