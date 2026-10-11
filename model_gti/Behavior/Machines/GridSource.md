---
type: StateDef
name: GridSource
subStates:
  - name: Live
    isInitial: true
    transitions:
      - {target: Live, accept: {payload: Set, as: m}, effect: {send: GridSample, via: out, payload: {v: "m.v", f: "m.f"}}}
---

Test environment: whatever voltage (pu) and frequency (Hz) a scenario sets is delivered as a sample to the software monitor and to the hardware trip. Grid loss is a sample with zero voltage.
