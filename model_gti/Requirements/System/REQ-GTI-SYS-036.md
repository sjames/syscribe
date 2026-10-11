---
type: Requirement
id: REQ-GTI-SYS-036
name: "Over-temperature derating and shutdown"
status: approved
reqDomain: system
verificationMethod: test
derivedFrom:
  - REQ-GTI-STK-003
  - REQ-GTI-STK-004
breakdownAdr: ADR-GTI-REQ-001
derivedFromSafetyGoal: SG-GTI-004
silLevel: 1
---

Above 45 °C ambient or heatsink limit the inverter **shall** derate output linearly and **shall** shut down at the maximum allowed temperature.
