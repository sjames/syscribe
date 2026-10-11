---
type: Requirement
id: REQ-GTI-SW-004
name: "Arc-fault signal analysis"
status: approved
reqDomain: software
verificationMethod: test
derivedFrom:
  - REQ-GTI-SYS-032
breakdownAdr: ADR-GTI-REQ-001
derivedFromSafetyGoal: SG-GTI-002
silLevel: 2
tags: [reg-unverified]
appliesWhen: Features::Protection::Afci
---

The arc-fault analyser **shall** classify the high-frequency current signature of each input and demand interruption when an arc is recognised.
