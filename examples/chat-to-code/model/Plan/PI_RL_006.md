---
type: PlanningItem
blockedBy: []
id: PI-RL-006
itemType: task
name: Make the limiter thread-safe
parent: PI-RL-001
status: done
assignedTo: claude
evidence:
- ref: TC-RL-004
- ref: ADR-RL-004
- path: ../src/ratelimit.py
---

Decide locking granularity (one lock or one per key), which ADR-RL-003 left open, and record it as a new ADR. Then add the locking so concurrent `try_acquire` and `acquire` calls never admit more than the bucket allows. `acquire` must not hold a lock while sleeping.

Satisfies REQ-RL-007; makes TC-RL-004 and the concurrent part of TC-RL-006 pass.

