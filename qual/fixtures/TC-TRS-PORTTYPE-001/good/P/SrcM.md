---
type: StateDef
name: SrcM
variables:
  - {name: n, value: "0"}
subStates:
  - name: A
    isInitial: true
    transitions:
      - {target: A, accept: Tick, effect: {send: Sample, via: out, payload: {v: "3 V", ok: "true"}}}
      - {target: A, accept: {payload: Trip, via: out}, assign: "n := n + 1"}
---
