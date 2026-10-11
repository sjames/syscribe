---
type: StateDef
name: Sensor
subStates:
  - name: Pass
    isInitial: true
    transitions:
      - {target: Pass, accept: {payload: GridSample, via: inp, as: s}, effect: {send: GridSample, via: out, payload: {v: "s.v", f: "s.f"}}}
---

Grid voltage and frequency sensing chain: forwards each sample (isolation amplifier, ADC and anti-alias filter are not modelled).
