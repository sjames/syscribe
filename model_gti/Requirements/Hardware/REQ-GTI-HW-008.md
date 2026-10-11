---
type: Requirement
id: REQ-GTI-HW-008
name: "Independent voltage sensing for the hardware trip"
status: approved
reqDomain: hardware
verificationMethod: test
derivedFrom:
  - REQ-GTI-SYS-039
breakdownAdr: ADR-GTI-REQ-001
derivedFromSafetyGoal: SG-GTI-001
silLevel: 2
---

The hardware trip path **shall** use a grid-voltage sensing chain that shares no sensor, amplifier or reference with the chain read by the microcontroller, so that one sensing fault cannot blind both disconnection paths. Rationale: fault-injection campaign result FM-GTI-009 (ADR-GTI-SAFE-001).
