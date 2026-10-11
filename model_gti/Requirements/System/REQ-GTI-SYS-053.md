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
---

Every remote command that changes behaviour **shall** be authenticated and integrity-protected; an unauthenticated command **shall** be rejected and logged.
