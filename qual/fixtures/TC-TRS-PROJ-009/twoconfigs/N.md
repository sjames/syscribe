---
type: StateDef
name: N
testExempt: fixture exercises parameters, not coverage
variables:
  - {name: x, value: "0"}
subStates:
  - {name: A, isInitial: true, transitions: [{target: B, accept: Go, guard: "x < {{Features::Profile.limit}}"}]}
  - {name: B, isFinal: true}
---
