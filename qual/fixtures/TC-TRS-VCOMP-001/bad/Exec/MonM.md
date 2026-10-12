---
type: StateDef
name: MonM
subStates:
  - name: A
    isInitial: true
    transitions:
      - {target: A, accept: {payload: Sample, via: samples}, effect: {send: Trip, via: out}}
---
