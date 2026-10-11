---
type: StateDef
name: HwTrip
subStates:
  - name: Armed
    isInitial: true
    transitions:
      - {target: Latched, accept: {payload: GridSample, via: samples, as: s}, guard: "s.v > 1.2 or s.v < 0.45", effect: {send: Trip, via: out}}
  - name: Latched
    transitions:
      - {target: Armed, accept: {payload: GridSample, via: samples, as: s}, guard: "s.v <= 1.2 and s.v >= 0.45", effect: {send: Rearm, via: out}}
---

Independent hardware trip path: comparators on the grid voltage open the second relay and disable the gate drivers without involving the STM32 firmware (REQ-GTI-SYS-037, REQ-GTI-SYS-038). It latches until the voltage is back in range, then re-arms by itself (the relay stays open until the supervisor closes it). Voltage window only; the thresholds are those of the fast stage of the US profile and are **unverified**.
