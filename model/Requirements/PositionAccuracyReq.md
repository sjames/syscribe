---
type: Requirement
id: REQ-UAV-NAV-001
title: "GPS position accuracy ≤ 1.5 m CEP under nominal GNSS"
status: approved
reqDomain: hardware
derivedFrom:
  - REQ-UAV-PERF-000
breakdownAdr: Decisions::PerformanceDecompositionADR
tags:
  - navigation
  - gps
---

The UAV GPS navigation system shall maintain position accuracy of ±1.5 m CEP under nominal GNSS conditions with at least 6 visible satellites.

## Rationale

Precision waypoint following for survey missions requires sub-2 m positional accuracy to maintain flight-line spacing within acceptable bounds.

## Notes

Allocation: `UAV::Avionics::GPSReceiver`.
CEP = Circular Error Probable (50th percentile radial error).
