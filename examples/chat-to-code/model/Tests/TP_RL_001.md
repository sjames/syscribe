---
type: TestPlan
demonstrates:
- REQ-RL-001
- REQ-RL-002
- REQ-RL-003
id: TP-RL-001
name: Rate limiter pytest suite
scope: regression
status: active
testCases:
- TC-RL-001
- TC-RL-002
- TC-RL-003
- TC-RL-004
- TC-RL-005
- TC-RL-006
---

## Objectives

Show that the `ratelimit` module meets its leaf requirements (REQ-RL-004 to REQ-RL-008) and the stakeholder requirements REQ-RL-001 to REQ-RL-003, the latter through the end-to-end test TC-RL-006.

The suite mixes levels (L1 unit, L2 thread-safety, L3 end-to-end); it is run in full on every change, hence scope `regression`.

## Environment

- Runner: pytest, invoked as `pytest tests/test_ratelimit.py`.
- Code under test: `src/ratelimit.py`.
- Time: TC-RL-001 to TC-RL-005 use a fake `Clock` (ADR-RL-003) whose `sleep(s)` advances `now()` instead of waiting, so they do not depend on real time.
- TC-RL-004 uses real threads with a frozen fake clock.
- TC-RL-006 uses the real `SystemClock` and takes about half a second in total.
- Python: 3.10 or later, standard library only (plus pytest as a dev tool).

## Entry criteria

`src/ratelimit.py` exposes the API in ADR-RL-003.

## Exit criteria

All six test cases pass.
