---
type: StateDef
name: MonM
variables:
  - {name: last, value: "0 V"}
subStates:
  - name: A
    isInitial: true
    transitions:
      - {target: A, accept: {payload: Sample, via: inp, as: s}, guard: "s.v > 2 V and s.okay", assign: "last := s.ok", effect: {send: Trip, via: inp}}
---
