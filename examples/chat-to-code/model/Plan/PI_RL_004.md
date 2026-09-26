---
type: PlanningItem
blockedBy: []
id: PI-RL-004
itemType: task
name: Implement clock, decision and non-blocking limiting
parent: PI-RL-001
status: done
assignedTo: claude
evidence:
- ref: TC-RL-001
- ref: TC-RL-002
- path: ../src/ratelimit.py
---

In `src/ratelimit.py`: `Clock` protocol, `SystemClock`, `Decision`, the per-key token bucket (lazy refill from `Clock.now()`, capped at capacity, starting full) and `RateLimiter.try_acquire`.

Satisfies REQ-RL-004 and REQ-RL-005; makes TC-RL-001 and TC-RL-002 pass.

