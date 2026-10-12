---
type: StateDef
name: PerturbationCheck
variables:
  - {name: pwm, value: "0"}
  - {name: iInj, value: "0 SI::A"}
  - {name: v0, value: "0"}
subStates:
  - name: Idle
    isInitial: true
    transitions:
      - {target: Baseline, accept: {after: "1 s"}, guard: "pwm == 1", effect: {send: SampleReq, via: req}}
      - {target: Idle, accept: {after: "1 s"}, guard: "pwm == 0"}
  - name: Baseline
    transitions:
      - {target: Settling, accept: {payload: Probe, via: samples, as: s}, assign: "v0 := s.v; iInj := 0.2 SI::A"}
      - {target: Lost, accept: {after: "100 ms"}}
  - name: Settling
    transitions:
      - {target: Measure, accept: {after: "100 ms"}, effect: {send: SampleReq, via: req}}
      - {target: Idle, accept: {payload: GridSample, via: samples}, assign: "iInj := 0 SI::A"}
  - name: Measure
    transitions:
      - {target: Idle, accept: {payload: Probe, via: samples, as: s}, guard: "(v0 - s.v) * (230 SI::V) >= iInj * (0.1 SI::ohm) and (v0 - s.v) * (230 SI::V) <= iInj * (5 SI::ohm)", assign: "iInj := 0 SI::A"}
      - {target: Lost, accept: {payload: Probe, via: samples, as: s}, guard: "(v0 - s.v) * (230 SI::V) < iInj * (0.1 SI::ohm) or (v0 - s.v) * (230 SI::V) > iInj * (5 SI::ohm)"}
      - {target: Idle, accept: {payload: GridSample, via: samples}, assign: "iInj := 0 SI::A"}
      - {target: Lost, accept: {after: "100 ms"}}
  - name: Lost
    entryAction: {send: GridLost, via: trip}
    transitions:
      - {target: Idle, accept: {after: "1 s"}, assign: "iInj := 0 SI::A"}
---

Perturbation-response check of the protection manager (`REQ-GTI-SW-008`). Once a second, while the bridge is switching (`pwm`), it takes a baseline voltage, injects a small current (0.2 A, about 1 % of rated current), waits 100 ms for the grid to settle and measures again. The change in voltage divided by the injected current is the grid impedance it sees. A connected grid answers with between 0.1 Ω (a quarter of the nominal 0.4 Ω) and 5 Ω (the weak-grid limit). Anything outside that range, or no answer at all, means the grid is lost, and the check reports `GridLost` to the supervisor. Below the range the response is missing, as when both sensing chains read a fixed value; above it the grid is islanded or too weak. The check does not rely on the absolute value of the voltage, so it still works when both sensing chains read a plausible but wrong value (`REQ-GTI-SYS-045`). A grid event during the measurement (an unrequested `GridSample`) discards that cycle. The limits are illustrative and **unverified**.
