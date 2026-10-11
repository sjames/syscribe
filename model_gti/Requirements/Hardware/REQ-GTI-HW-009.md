---
type: Requirement
id: REQ-GTI-HW-009
name: "Hardware trip actuation"
status: approved
reqDomain: hardware
verificationMethod: test
derivedFrom:
  - REQ-GTI-SYS-039
breakdownAdr: ADR-GTI-REQ-001
derivedFromSafetyGoal: SG-GTI-001
silLevel: 2
---

On a gross grid-voltage loss the hardware trip **shall** open the second grid relay and disable the gate drivers without involving the microcontroller, and **shall** re-arm itself when the voltage has returned.
