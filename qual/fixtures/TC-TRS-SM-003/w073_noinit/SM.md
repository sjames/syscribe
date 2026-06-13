---
type: StateDef
name: SM
subStates:
  - name: A
    transitions:
      - target: B
  - name: B
    transitions:
      - target: A
---

No substate is `isInitial` — W073. A↔B is otherwise connected (no dead/trap).
