---
type: PlanningItem
blockedBy: []
id: PI-RL-003
itemType: task
name: Write the pytest suite
parent: PI-RL-001
status: done
assignedTo: claude
evidence:
- ref: TP-RL-001
- path: ../tests/test_ratelimit.py
---

Write `tests/test_ratelimit.py` with a `FakeClock` (`now()`, and `sleep(s)` that records `s` and advances time) and one test function per scenario in TC-RL-001 to TC-RL-006, using exactly the function names listed in each test case's `testFunctions`.

Written against the API in ADR-RL-003, before the implementation, so the tests fail first.

