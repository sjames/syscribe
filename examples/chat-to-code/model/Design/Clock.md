---
type: PartDef
name: Clock
satisfies:
- REQ-RL-006
---

Source of time for the limiter: `now()` (monotonic seconds) and `sleep(seconds)` (see ADR-RL-003).

Exists as a separate part so tests can substitute a fake clock and run deterministically.
