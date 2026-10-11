---
type: Requirement
id: REQ-GTI-HW-004
name: "Arc-fault sensing front end"
status: approved
reqDomain: hardware
verificationMethod: test
derivedFrom:
  - REQ-GTI-SYS-032
breakdownAdr: ADR-GTI-REQ-001
derivedFromSafetyGoal: SG-GTI-002
silLevel: 2
tags: [reg-unverified]
appliesWhen: Features::Protection::Afci
---

A dedicated wide-band current sensing path **shall** deliver the high-frequency signal of each tracker input to the MCU converter inputs.
