---
type: Requirement
id: REQ-UAV-FC-001
name: "Flight controller shall detect single sensor failure within 50 ms"
status: approved
asilLevel: C
verificationMethod: test
reqDomain: software
derivedFrom:
  - REQ-UAV-SAFE-000
breakdownAdr: Decisions::SafetyDecompositionADR
tags:
  - safety
  - fault-detection
  - flight-controller
---

The flight controller shall detect and respond to any single sensor failure within 50 ms of the failure onset. Response shall include mode transition to a degraded operating mode and emission of a fault telemetry event.

## Rationale

DO-178C DAL A requires bounded fault detection and isolation latency. A 50 ms detection window ensures the autopilot can initiate corrective action before attitude error exceeds recoverable bounds.

## Notes

Allocation: `UAV::Avionics::FlightController`.
HIL test: inject sensor failure signal and measure time to FC mode-transition command.
