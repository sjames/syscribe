---
type: StateDef
name: SrcM
variables:
  - {name: n, value: "0"}
subStates:
  - name: A
    isInitial: true
    transitions:
      - {target: A, accept: Tick, effect: {send: Sample, via: out, payload: {v: "3 A", ok: "true", x: "1"}}}
      - {target: A, accept: Tock, effect: {send: Bogus, via: out}}
      - {target: A, accept: Tuck, effect: {send: Trip, via: out}}
      - {target: A, accept: {payload: Trip, via: out}, assign: "n := n + 1"}
---
