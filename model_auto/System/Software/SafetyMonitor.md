---
type: PartDef
name: Safety Monitor
domain: software
asilLevel: D
satisfies:
  - REQ-ENG-SAFE-001
  - REQ-ENG-SAFE-004
  - REQ-ENG-SAFE-005
features:
  - name: monitorCycleMs
    type: ScalarValues::Integer
    unit: ms
  - name: tpsDivergenceThresholdPct
    type: ScalarValues::Real
---

Safety monitoring software component (ASIL D). Supervises all safety-relevant
inputs and function outputs. Implements the following checks:

- **TPS dual-track divergence** — detects > 5 % deviation between tracks
- **Throttle position vs. command** — detects actuator stuck-at fault
- **Pedal vs. brake conflict** — detects simultaneous full-throttle and brake demand
- **Watchdog timeout** — communicates with the hardware watchdog timer

On any fault detection, the component asserts a fault signal to ThrottleControl
and FuelControl, logs a DTC, and triggers a controlled engine shutdown if the
fault persists for more than 200 ms.
