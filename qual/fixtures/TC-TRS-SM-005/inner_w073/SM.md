---
type: StateDef
name: SM
subStates:
  - name: Active
    isInitial: true
    subStates:
      - name: X
        transitions:
          - target: Y
      - name: Y
        transitions:
          - target: X
    transitions:
      - target: Done
  - name: Done
    isFinal: true
---

The top level is well-formed (`Active` initial → `Done` final), but the inner region of the
composite `Active` has no `isInitial` substate — W073 naming region `Active`.
