---
type: Requirement
id: REQ-UAV-ENDUR-001
name: "Minimum 25-minute flight endurance under nominal conditions"
status: approved
reqDomain: hardware
derivedFrom:
  - REQ-UAV-PERF-000
breakdownAdr: Decisions::PerformanceDecompositionADR
tags:
  - endurance
  - mission-planning
---

The UAV shall maintain continuous flight for a minimum of 25 minutes under nominal payload and wind conditions.

## Rationale

Mission planning analysis requires a minimum endurance budget to cover the standard survey area with adequate margins for transit and contingency holds.

## Notes

Assumes: wind speed ≤ 10 m/s, payload mass ≤ 0.5 kg.
Allocation: `UAV::Power::BatteryPack`.
