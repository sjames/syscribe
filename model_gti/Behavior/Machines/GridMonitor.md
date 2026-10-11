---
type: StateDef
name: GridMonitor
subStates:
  - name: Unknown
    isInitial: true
    transitions:
      - {target: Normal, accept: {payload: GridSample, via: samples, as: s}, guard: "s.v <= {{Features::GridProfile.overVoltagePu}} and s.v >= {{Features::GridProfile.underVoltagePu}} and s.f <= {{Features::GridProfile.overFreqHz}} and s.f >= {{Features::GridProfile.underFreqHz}}", effect: {send: GridGood, via: out}}
      - {target: Excursion, accept: {payload: GridSample, via: samples, as: s}, guard: "s.v > {{Features::GridProfile.overVoltagePu}} or s.v < {{Features::GridProfile.underVoltagePu}} or s.f > {{Features::GridProfile.overFreqHz}} or s.f < {{Features::GridProfile.underFreqHz}}"}
  - name: Normal
    transitions:
      - {target: Excursion, accept: {payload: GridSample, via: samples, as: s}, guard: "s.v > {{Features::GridProfile.overVoltagePu}} or s.v < {{Features::GridProfile.underVoltagePu}} or s.f > {{Features::GridProfile.overFreqHz}} or s.f < {{Features::GridProfile.underFreqHz}}"}
  - name: Excursion
    transitions:
      - {target: Normal, accept: {payload: GridSample, via: samples, as: s}, guard: "s.v <= {{Features::GridProfile.overVoltagePu}} and s.v >= {{Features::GridProfile.underVoltagePu}} and s.f <= {{Features::GridProfile.overFreqHz}} and s.f >= {{Features::GridProfile.underFreqHz}}"}
      - {target: Tripped, accept: {after: "{{Features::GridProfile.clearingMs}} ms"}, effect: {send: GridLost, via: out}}
  - name: Tripped
    transitions:
      - {target: Normal, accept: {payload: GridSample, via: samples, as: s}, guard: "s.v <= {{Features::GridProfile.reconnectVoltageMaxPu}} and s.v >= {{Features::GridProfile.reconnectVoltageMinPu}} and s.f <= {{Features::GridProfile.reconnectFreqMaxHz}} and s.f >= {{Features::GridProfile.reconnectFreqMinHz}}", effect: {send: GridGood, via: out}}
---

Software grid monitor; every limit and time is a parameter of the active grid profile (`Features::GridProfile`), so one machine serves both regions (run with `--config`). Values below are those of the bound Configuration. For reference, the profile when voltage (per unit) or frequency (Hz) leaves the window [{{Features::GridProfile.underVoltagePu}}, {{Features::GridProfile.overVoltagePu}}] pu / [{{Features::GridProfile.underFreqHz}}, {{Features::GridProfile.overFreqHz}}] Hz for {{Features::GridProfile.clearingMs}} ms it reports `GridLost`; when the grid is back inside the window it reports `GridGood`; after a trip the grid counts as good only inside the narrower reconnection window [{{Features::GridProfile.reconnectVoltageMinPu}}, {{Features::GridProfile.reconnectVoltageMaxPu}}] pu / [{{Features::GridProfile.reconnectFreqMinHz}}, {{Features::GridProfile.reconnectFreqMaxHz}}] Hz (RM-08). The first sample after power-up is classified too, so a dead grid at power-up is reported. Values are the fast-stage defaults of the regulatory matrix (RM-03 to RM-06) and are **unverified**. Only the fast stage is modelled; the slower stages of the trip table are not.
