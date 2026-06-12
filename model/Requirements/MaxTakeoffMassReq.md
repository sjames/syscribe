---
type: Requirement
id: REQ-UAV-MASS-001
name: "Maximum take-off mass shall not exceed 5 kg"
status: approved
reqDomain: system
derivedFrom:
  - REQ-UAV-REG-000
breakdownAdr: Decisions::PerformanceDecompositionADR
tags:
  - mass
  - regulatory
---

The UAV total take-off mass shall not exceed 5.0 kg, including payload and battery, to remain within the sub-5 kg regulatory category.

## Rationale

Regulatory compliance (EASA category Open A3 / sub-5 kg) requires mass at or below 5 kg at all times. Exceeding this boundary triggers a higher certification category.

## Notes

Allocation: `UAV::UAVSystem`.
Verified by mass budget calculation `Calculations::TotalMassCalc`.
