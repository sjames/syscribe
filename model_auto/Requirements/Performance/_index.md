---
type: Package
name: Performance
---

Performance requirements govern the dynamic response and emissions compliance of the engine
management system. They are derived from `REQ-ENG-PERF-000` via `ADR-ENG-PERF-001`, which
decomposes the parent into two independent performance dimensions.

## Requirements in this package

| ID | Title | Verification |
|---|---|---|
| `REQ-ENG-PERF-000` | Engine ECU shall meet throttle response and fuel efficiency targets | analysis |
| `REQ-ENG-PERF-001` | Throttle position shall reach commanded value within 150 ms | test (L3) |
| `REQ-ENG-PERF-002` | Lambda closed-loop shall converge to target within 500 ms | test (L3) |

## Engineering context

Throttle response (REQ-ENG-PERF-001) is the primary driver for the PID cycle time (10 ms) and
anti-windup design of `ThrottleControl`. The 150 ms budget includes actuator mechanics, sensor
feedback lag, and control loop latency.

Lambda accuracy (REQ-ENG-PERF-002) drives the PI correction bandwidth in `FuelControl`. The
500 ms convergence budget applies after a steady-state load step; cold-start enrichment operates
in open-loop until the lambda sensor reaches operating temperature (≥ 300 °C heater pre-condition).
