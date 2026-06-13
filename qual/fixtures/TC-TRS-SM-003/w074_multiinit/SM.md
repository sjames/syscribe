---
type: StateDef
name: SM
subStates:
  - name: A
    isInitial: true
    transitions:
      - target: B
  - name: B
    isInitial: true
    transitions:
      - target: A
---

Both `A` and `B` are `isInitial` — W074.
