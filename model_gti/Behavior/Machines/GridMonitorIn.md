---
type: StateDef
name: GridMonitorIn
subStates:
  - name: Normal
    isInitial: true
    transitions:
      - {target: Excursion, accept: {payload: GridSample, via: samples, as: s}, guard: "s.v > 1.1 or s.v < 0.85 or s.f > 50.5 or s.f < 47.5"}
  - name: Excursion
    transitions:
      - {target: Normal, accept: {payload: GridSample, via: samples, as: s}, guard: "s.v <= 1.1 and s.v >= 0.85 and s.f <= 50.5 and s.f >= 47.5"}
      - {target: Tripped, accept: {after: "200 ms"}, effect: {send: GridLost, via: out}}
  - name: Tripped
    transitions:
      - {target: Normal, accept: {payload: GridSample, via: samples, as: s}, guard: "s.v <= 1.1 and s.v >= 0.85 and s.f <= 50.5 and s.f >= 47.5", effect: {send: GridGood, via: out}}
---

Software grid monitor of the India profile: when voltage (per unit) or frequency (Hz) leaves the window [0.85, 1.1] pu / [47.5, 50.5] Hz for 200 ms it reports `GridLost`; when the grid is back inside the window it reports `GridGood`. Values are the fast-stage defaults of the regulatory matrix (RM-03 to RM-06) and are **unverified**. Only the fast stage is modelled; the slower stages of the trip table are not.
