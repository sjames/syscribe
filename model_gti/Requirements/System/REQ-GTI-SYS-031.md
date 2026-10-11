---
type: Requirement
id: REQ-GTI-SYS-031
name: "Residual-current monitoring"
status: approved
reqDomain: system
verificationMethod: test
derivedFrom:
  - REQ-GTI-STK-003
breakdownAdr: ADR-GTI-REQ-001
derivedFromSafetyGoal: SG-GTI-003
silLevel: 2
---

The inverter **shall** disconnect within the times of IEC 62109-2 when residual current exceeds 300 mA continuously or rises in steps of 30, 60 or 150 mA `[secondary, RM-17]`.
