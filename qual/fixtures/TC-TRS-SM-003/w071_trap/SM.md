---
type: StateDef
name: SM
subStates:
  - name: Init
    isInitial: true
    transitions:
      - target: Run
  - name: Run
    transitions:
      - target: Trap
  - name: Trap
---

`Trap` has no outgoing transition and is not final — W071.
