---
type: StateDef
name: SM
subStates:
  - name: A
    isInitial: true
    transitions:
      - target: B
      - target: Ghost
  - name: B
    isFinal: true
---

`A` transitions to `Ghost`, which is not a state anywhere in this machine and resolves to no
model element — W076. The `A` → `B` edge is fine.
