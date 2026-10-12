---
type: StateDef
name: StimM
subStates:
  - name: Idle
    isInitial: true
    transitions:
      - {target: Idle, accept: Go, effect: {send: Sample, via: out}}
---
