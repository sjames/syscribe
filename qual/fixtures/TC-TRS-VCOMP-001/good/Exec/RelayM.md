---
type: StateDef
name: RelayM
subStates:
  - name: Closed
    isInitial: true
    transitions:
      - {target: Open, accept: {payload: Trip, via: cmd}}
  - {name: Open}
---
