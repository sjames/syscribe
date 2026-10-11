---
type: Requirement
id: REQ-GTI-SW-005
name: "Rapid-shutdown control"
status: approved
reqDomain: software
verificationMethod: test
derivedFrom:
  - REQ-GTI-SYS-033
breakdownAdr: ADR-GTI-REQ-001
derivedFromSafetyGoal: SG-GTI-005
silLevel: 1
tags: [reg-unverified]
appliesWhen: Features::Protection::RapidShutdown
---

On initiator or AC loss the supervisor **shall** stop the boost stages and signal the module-level devices.
