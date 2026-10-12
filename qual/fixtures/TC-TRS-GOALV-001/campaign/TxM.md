---
type: StateDef
name: TxM
variables:
  - {name: level, value: "7"}
subStates:
  - name: Idle
    isInitial: true
    transitions:
      - {target: Idle, accept: Tick, effect: {send: Cmd, via: out, payload: {n: "level"}}}
---
