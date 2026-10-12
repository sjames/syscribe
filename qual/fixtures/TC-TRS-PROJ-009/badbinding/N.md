---
type: StateDef
name: N
variables:
  - {name: x, value: "0"}
subStates:
  - {name: A, isInitial: true, transitions: [{target: B, accept: Go, guard: "x < {{Features::Profile.limit}}"}]}
  - {name: B, isFinal: true}
---
