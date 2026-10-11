---
type: StateDef
name: GridMonitorUs
subStates:
  - name: Unknown
    isInitial: true
    transitions:
      - {target: Normal, accept: {payload: GridSample, via: samples, as: s}, guard: "s.v <= 1.2 and s.v >= 0.45 and s.f <= 62.0 and s.f >= 56.5", effect: {send: GridGood, via: out}}
      - {target: Excursion, accept: {payload: GridSample, via: samples, as: s}, guard: "s.v > 1.2 or s.v < 0.45 or s.f > 62.0 or s.f < 56.5"}
  - name: Normal
    transitions:
      - {target: Excursion, accept: {payload: GridSample, via: samples, as: s}, guard: "s.v > 1.2 or s.v < 0.45 or s.f > 62.0 or s.f < 56.5"}
  - name: Excursion
    transitions:
      - {target: Normal, accept: {payload: GridSample, via: samples, as: s}, guard: "s.v <= 1.2 and s.v >= 0.45 and s.f <= 62.0 and s.f >= 56.5"}
      - {target: Tripped, accept: {after: "160 ms"}, effect: {send: GridLost, via: out}}
  - name: Tripped
    transitions:
      - {target: Normal, accept: {payload: GridSample, via: samples, as: s}, guard: "s.v <= 1.05 and s.v >= 0.917 and s.f <= 60.1 and s.f >= 59.5", effect: {send: GridGood, via: out}}
---

Software grid monitor of the United States profile: when voltage (per unit) or frequency (Hz) leaves the window [0.45, 1.2] pu / [56.5, 62.0] Hz for 160 ms it reports `GridLost`; when the grid is back inside the window it reports `GridGood`; after a trip the grid counts as good only inside the narrower reconnection window [0.917, 1.05] pu / [59.5, 60.1] Hz (RM-08). The first sample after power-up is classified too, so a dead grid at power-up is reported. Values are the fast-stage defaults of the regulatory matrix (RM-03 to RM-06) and are **unverified**. Only the fast stage is modelled; the slower stages of the trip table are not.
