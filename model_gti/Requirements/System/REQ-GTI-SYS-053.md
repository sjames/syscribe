---
type: Requirement
id: REQ-GTI-SYS-053
name: "Authenticated remote commands"
status: approved
reqDomain: system
verificationMethod: test
derivedFrom:
  - REQ-GTI-STK-005
  - REQ-GTI-STK-003
breakdownAdr: ADR-GTI-REQ-001
derivedFromCybersecurityGoal: CSG-GTI-001
calLevel: CAL4
---

Every remote command that changes behaviour (including Modbus writes) **shall** be authenticated and integrity-protected; read-only telemetry may be unauthenticated; an unauthenticated command **shall** be rejected and logged.
