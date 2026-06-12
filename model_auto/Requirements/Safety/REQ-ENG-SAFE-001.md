---
type: Requirement
id: REQ-ENG-SAFE-001
typedBy: Requirements::ASILRequirementDef
name: Safety monitor shall detect all safety faults within 100 ms
status: approved
reqDomain: software
asilLevel: D
verificationMethod: test
derivedFrom:
  - REQ-ENG-SAFE-000
breakdownAdr: ADR-ENG-SAFE-001
derivedFromSafetyGoal: SG-ENG-001
---

The safety monitor software component **shall** detect any single-point safety
fault (TPS divergence, throttle position vs. command deviation, pedal-brake
conflict, watchdog communication failure) within 100 ms of fault onset and
assert the fail-safe output to the throttle control and fuel control components.

Compliance evidence: hardware-in-the-loop fault injection tests covering all
defined fault modes (ISO 26262-6 §8.4.4).
