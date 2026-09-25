---
type: StateDef
name: SM
status: draft
subStates:
  - name: Idle
    isInitial: true
  - name: Running
    isFinal: true
transitions:
  - target: Running
---

Draft machine: the incomplete top-level transition is draft-suppressed.
