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
      - target: Init
  - name: Dead
    transitions:
      - target: Run
---

`Dead` has no incoming transition and is not initial — W070.
