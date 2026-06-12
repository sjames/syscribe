---
type: Requirement
id: REQ-UAV-COMM-001
name: "Telemetry data link range ≥ 5 km line of sight"
status: approved
reqDomain: software
derivedFrom:
  - REQ-UAV-PERF-000
breakdownAdr: Decisions::PerformanceDecompositionADR
tags:
  - communication
  - telemetry
---

The command and telemetry data link shall maintain bidirectional connectivity at ranges up to 5 km line of sight under standard atmospheric conditions, with a packet error rate not exceeding 1 %.

## Rationale

Mission radius is constrained by the data link range. A 5 km range supports the planned survey corridor with margin for operator repositioning.

## Notes

Allocation: `UAV::Avionics::FlightController` (telemetry transmitter).
Ground-side: `GroundStation::GroundControlStation`.
