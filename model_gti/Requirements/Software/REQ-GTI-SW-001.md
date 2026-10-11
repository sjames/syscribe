---
type: Requirement
id: REQ-GTI-SW-001
name: "Voltage and frequency trip logic"
status: approved
reqDomain: software
verificationMethod: test
derivedFrom:
  - REQ-GTI-SYS-007
breakdownAdr: ADR-GTI-REQ-001
tags: [reg-unverified]
---

The protection manager **shall** evaluate every grid sample against the trip table of the active profile and open the relays when a limit is exceeded for longer than the clearing time.
