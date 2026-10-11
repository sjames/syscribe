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

Above 45 °C ambient or heatsink limit the inverter **shall** derate output linearly from 100 % at 45 °C to at least 50 % at 60 °C ambient and **shall** shut down when the heatsink reaches its maximum allowed temperature (value set by the thermal design).
