---
type: StateDef
name: SM
subStates:
  - name: Init
    isInitial: true
    entryAction: DoThing
    transitions:
      - target: Done
        effect: DoThing
  - name: Done
    isFinal: true
---

`entryAction` and the transition `effect` both reference the resolvable `DoThing` action —
no W079. The machine is well-formed.
