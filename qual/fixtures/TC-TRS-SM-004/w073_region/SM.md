---
type: StateDef
name: SM
isParallel: true
subStates:
  - name: RA
    subStates:
      - name: A1
        transitions:
          - target: A2
      - name: A2
        transitions:
          - target: A1
  - name: RB
    subStates:
      - name: B1
        isInitial: true
        isFinal: true
---

Region `RA` has no `isInitial` substate — W073 naming RA. `RB` is well-formed; two regions
so no W078; no cross-region transitions.
