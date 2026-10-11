---
type: Requirement
id: REQ-GTI-SYS-033
name: "Rapid shutdown of array conductors"
status: approved
reqDomain: system
verificationMethod: test
derivedFrom:
  - REQ-GTI-STK-003
breakdownAdr: ADR-GTI-REQ-001
derivedFromSafetyGoal: SG-GTI-005
silLevel: 1
tags: [reg-unverified]
appliesWhen: Features::Protection::RapidShutdown
---

On loss of AC or on an initiator signal the inverter **shall** reduce voltage on controlled array conductors to 30 V within 30 s `[unverified, RM-13]`.
