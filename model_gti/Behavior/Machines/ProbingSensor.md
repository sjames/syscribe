---
type: StateDef
name: ProbingSensor
subStates:
  - name: Pass
    isInitial: true
    transitions:
      - {target: Pass, accept: {payload: GridSample, via: inp, as: s}, effect: {send: GridSample, via: out, payload: {v: "s.v", f: "s.f"}}}
      - {target: Pass, accept: {payload: Probe, via: inp, as: s}, effect: {send: Probe, via: out, payload: {v: "s.v"}}}
---

The software sensing chain: as `Sensor`, and it also forwards each probe of the perturbation check, so the check measures on the same chain as the monitor.
