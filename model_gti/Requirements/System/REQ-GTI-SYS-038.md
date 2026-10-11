---
type: Requirement
id: REQ-GTI-SYS-038
name: "Single-fault-tolerant grid disconnection"
status: approved
reqDomain: system
verificationMethod: test
derivedFrom:
  - REQ-GTI-STK-003
breakdownAdr: ADR-GTI-REQ-001
derivedFromSafetyGoal: SG-GTI-001
silLevel: 2
---

Two grid relays in series, each independently capable of isolating the inverter, **shall** be provided; a welded relay **shall** be detected before the next connection and prevent it.
