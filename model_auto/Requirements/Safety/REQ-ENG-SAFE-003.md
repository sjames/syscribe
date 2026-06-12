---
type: Requirement
id: REQ-ENG-SAFE-003
name: Engine stall monitor shall detect CPS loss and initiate deceleration
status: approved
reqDomain: software
asilLevel: B
verificationMethod: test
derivedFrom:
  - REQ-ENG-SAFE-000
breakdownAdr: ADR-ENG-SAFE-001
derivedFromSafetyGoal: SG-ENG-002
---

The engine stall monitor **shall** detect loss of crankshaft position sensor
signal within two consecutive engine cycles and initiate a controlled
deceleration sequence (fuel cut, throttle to idle position) before the
engine speed drops below the stall threshold (400 rpm), while alerting the
driver via the instrument cluster warning light.
