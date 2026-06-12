---
type: Requirement
id: REQ-UAV-SAFE-001
name: "Autonomous safe landing on battery-critical or link-loss event"
status: approved
asilLevel: B
verificationMethod: test
reqDomain: software
derivedFrom:
  - REQ-UAV-SAFE-000
breakdownAdr: Decisions::SafetyDecompositionADR
tags:
  - safety
  - contingency
  - landing
---

The UAV shall execute an autonomous landing with a controlled descent rate not exceeding 3 m/s upon detection of a battery-critical event (charge ≤ 10 %) or loss of command link for more than 3 seconds.

## Rationale

An uncontrolled descent following loss of link or battery depletion risks injury to persons on the ground. Limiting descent rate to 3 m/s ensures ground impact energy remains within acceptable limits for the sub-5 kg class.

## Notes

Allocation: `UAV::Avionics::FlightController`.
Trigger condition implemented in `Behavior::FlightStates` state `fault`.
