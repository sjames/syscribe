---
type: StateDef
name: GridMonitorUs
subStates:
  - name: Normal
    isInitial: true
    transitions:
      - {target: Excursion, accept: {payload: GridSample, via: samples, as: s}, guard: "s.v > 1.2 or s.v < 0.45 or s.f > 62.0 or s.f < 56.5"}
  - name: Excursion
    transitions:
      - {target: Normal, accept: {payload: GridSample, via: samples, as: s}, guard: "s.v <= 1.2 and s.v >= 0.45 and s.f <= 62.0 and s.f >= 56.5"}
      - {target: Tripped, accept: {after: "160 ms"}, effect: {send: GridLost, via: out}}
  - name: Tripped
    transitions:
      - {target: Normal, accept: {payload: GridSample, via: samples, as: s}, guard: "s.v <= 1.2 and s.v >= 0.45 and s.f <= 62.0 and s.f >= 56.5", effect: {send: GridGood, via: out}}
---

Software grid monitor of the United States profile: when voltage (per unit) or frequency (Hz) leaves the window [0.45, 1.2] pu / [56.5, 62.0] Hz for 160 ms it reports `GridLost`; when the grid is back inside the window it reports `GridGood`. Values are the fast-stage defaults of the regulatory matrix (RM-03 to RM-06) and are **unverified**. Only the fast stage is modelled; the slower stages of the trip table are not.
