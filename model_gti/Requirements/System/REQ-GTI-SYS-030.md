---
type: Requirement
id: REQ-GTI-SYS-030
name: "Insulation resistance check before connecting"
status: approved
reqDomain: system
verificationMethod: test
derivedFrom:
  - REQ-GTI-STK-003
breakdownAdr: ADR-GTI-REQ-001
derivedFromSafetyGoal: SG-GTI-003
silLevel: 2
---

Before closing the grid relays, the inverter **shall** measure the insulation resistance of the array to ground and **shall not** connect when it is below the limit computed per IEC 62109-2 `[secondary, RM-16]`.
