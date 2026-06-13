---
type: StateDef
name: SM
status: draft
subStates:
  - name: A
    transitions:
      - target: B
  - name: B
    transitions:
      - target: A
---

Missing-initial machine but `status: draft` — W073 (and all W07x) suppressed.
