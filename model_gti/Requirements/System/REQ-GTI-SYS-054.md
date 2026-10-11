---
type: Requirement
id: REQ-GTI-SYS-054
name: "Signed firmware update"
status: approved
reqDomain: system
verificationMethod: test
derivedFrom:
  - REQ-GTI-STK-005
  - REQ-GTI-STK-006
breakdownAdr: ADR-GTI-REQ-001
derivedFromCybersecurityGoal: CSG-GTI-002
calLevel: CAL3
---

The inverter **shall** install only firmware signed by the manufacturer and **shall** refuse an older version than the installed one.
