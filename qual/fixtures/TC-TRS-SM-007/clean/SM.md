---
type: StateDef
name: SM
status: approved
subStates:
  - name: Idle
    isInitial: true
    transitions:
      - target: Running
  - name: Running
    transitions:
      - target: Stopped
  - name: Stopped
    isFinal: true
transitions:
  - source: Stopped
    target: Idle
---

Nested transitions omit `source:` (implicit), the top-level one names it — no W929.
