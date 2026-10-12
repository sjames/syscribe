---
type: StateDef
name: GridSource
variables:
  - {name: vSet, value: "1.0"}
  - {name: island, value: "0"}
  - {name: zGrid, value: "0.4 SI::ohm"}
  - {name: dv, value: "0 SI::V"}
subStates:
  - name: Live
    isInitial: true
    transitions:
      - {target: Live, accept: {payload: Set, as: m}, guard: "m.v >= 0.5", assign: "vSet := m.v; island := 0", effect: {send: GridSample, via: out, payload: {v: "m.v", f: "m.f"}}}
      - {target: Live, accept: {payload: Set, as: m}, guard: "m.v < 0.5", assign: "vSet := m.v; island := 1", effect: {send: GridSample, via: out, payload: {v: "m.v", f: "m.f"}}}
      - {internal: true, accept: Breaker, assign: "island := 1"}
      - {internal: true, accept: {payload: Impedance, as: m}, assign: "zGrid := m.z"}
      - {internal: true, accept: {payload: SampleReq, via: req}, effect: {send: Probe, via: probe, payload: {v: "vSet + dv / (230 SI::V)"}}}
---

Test environment: whatever voltage (pu) and frequency (Hz) a scenario sets is delivered as a sample to the software monitor and to the hardware trip. Grid loss is a sample with zero voltage; a voltage below half of nominal also means the grid breaker has opened (`island`), so the grid no longer holds the voltage against the inverter's current. `Breaker` opens the grid breaker with a balanced local load: the island keeps its voltage and frequency, so no sample changes and only the perturbation check can tell. `Impedance` sets the impedance of the connected grid (a weak grid). On a sample request from the perturbation check it answers with a `Probe`: the set voltage plus the response of the grid impedance plant (`dv`, from `Behavior::Plants::GridImpedance`) to the injected current, on the software sensing chain, the same path the monitor's samples take (the hardware trip does not take part in the check).
