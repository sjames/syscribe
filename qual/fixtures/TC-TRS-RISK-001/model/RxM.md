---
type: StateDef
name: RxM
variables:
  - {name: got, value: "0"}
subStates:
  - name: Waiting
    isInitial: true
    transitions:
      - {target: Done, accept: {payload: Cmd, via: inp, as: m}, assign: "got := m.n"}
  - {name: Done, isFinal: true}
---
