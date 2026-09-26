---
type: PlanningItem
blockedBy: []
id: PI-RL-005
itemType: task
name: Implement blocking acquire
parent: PI-RL-001
status: done
assignedTo: claude
evidence:
- ref: TC-RL-003
- path: ../src/ratelimit.py
---

Add `RateLimiter.acquire(key)`: try, and while refused call `clock.sleep(retry_after)` and try again.

Satisfies REQ-RL-006; makes TC-RL-003 pass.

