---
type: Requirement
id: REQ-GTI-HW-010
name: "Current sensing for the perturbation check"
status: approved
reqDomain: hardware
verificationMethod: test
derivedFrom:
  - REQ-GTI-SYS-045
breakdownAdr: ADR-GTI-REQ-001
derivedFromSafetyGoal: SG-GTI-001
silLevel: 2
---

The AC output current sensors **shall** resolve a perturbation of 0.5 % of rated current with the bandwidth the perturbation-response check needs, using a sensing path that shares no sensor or reference with the grid-voltage chains read by the microcontroller and the hardware trip.
