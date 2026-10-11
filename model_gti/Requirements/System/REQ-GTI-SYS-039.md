---
type: Requirement
id: REQ-GTI-SYS-039
name: "Independent hardware voltage trip"
status: approved
reqDomain: system
verificationMethod: test
derivedFrom:
  - REQ-GTI-STK-003
breakdownAdr: ADR-GTI-REQ-001
derivedFromSafetyGoal: SG-GTI-001
silLevel: 2
---

Independently of the microcontroller, a hardware path **shall** open one grid relay and disable the gate drivers when the grid voltage is lost, so that loss of the software path alone does not leave the grid energised.
