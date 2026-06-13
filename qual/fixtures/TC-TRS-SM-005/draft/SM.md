---
type: StateDef
name: SM
status: draft
subStates:
  - name: A
    isInitial: true
    transitions:
      - target: Ghost
  - name: B
    isFinal: true
---

Unresolved endpoint `Ghost`, but `status: draft` — W076 (and all W07x) suppressed.
