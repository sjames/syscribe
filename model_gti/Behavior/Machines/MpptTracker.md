---
type: StateDef
name: MpptTracker
variables:
  - {name: vRef, value: "600"}
  - {name: dir, value: "1"}
  - {name: pPrev, value: "0"}
  - {name: pBest, value: "0"}
  - {name: vBest, value: "600"}
subStates:
  - name: Tracking
    isInitial: true
    transitions:
      - {target: Tracking, accept: {payload: Sample, as: s}, guard: "s.p >= pPrev", assign: "pPrev := s.p; vRef := vRef + dir * 5"}
      - {target: Tracking, accept: {payload: Sample, as: s}, guard: "s.p < pPrev", assign: "dir := 0 - dir; pPrev := s.p; vRef := vRef + dir * 5"}
      - {target: Scanning, accept: {after: "600 s"}, assign: "vRef := 200; pBest := 0; vBest := 200"}
  - name: Scanning
    transitions:
      - {target: Scanning, accept: {payload: Sample, as: s}, guard: "vRef <= 850 and s.p > pBest", assign: "pBest := s.p; vBest := vRef; vRef := vRef + 50"}
      - {target: Scanning, accept: {payload: Sample, as: s}, guard: "vRef <= 850 and s.p <= pBest", assign: "vRef := vRef + 50"}
      - {target: Tracking, accept: {payload: Sample, as: s}, guard: "vRef > 850", assign: "vRef := vBest; pPrev := pBest; dir := 1"}
---

Perturb-and-observe tracker of one MPPT input with a periodic global scan (feature `Mppt::GlobalScan`, `REQ-GTI-SYS-022`, `REQ-GTI-SYS-023`). `Sample` carries the measured string power `p` (W) after each step. In `Tracking` the reference voltage `vRef` (V) moves 5 V per sample in the current direction and reverses when power falls. Every 600 s (the default of `scanPeriodS`, not wired to the feature parameter, see FINDINGS) the tracker sweeps 200–850 V in 50 V steps, remembers the best point and returns to it. The step sizes and the window are the unverified design defaults of REQ-GTI-SYS-021.
