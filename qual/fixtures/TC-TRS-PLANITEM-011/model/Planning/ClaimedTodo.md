---
type: PlanningItem
id: PI-P11-CLAIMED-001
name: "Todo item already claimed by agent-1"
status: todo
achieves: REQ-P11-001
claimedBy: agent-1
claimedAt: "2026-01-01T00:00:00Z"
---

Claimed by agent-1 with a stale claimedAt -- a claim by agent-2 must be refused; a re-claim
by agent-1 must succeed and refresh claimedAt.
