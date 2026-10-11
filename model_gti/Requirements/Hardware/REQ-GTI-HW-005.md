---
type: Requirement
id: REQ-GTI-HW-005
name: "Rapid-shutdown signalling driver"
status: approved
reqDomain: hardware
verificationMethod: test
derivedFrom:
  - REQ-GTI-SYS-033
breakdownAdr: ADR-GTI-REQ-001
derivedFromSafetyGoal: SG-GTI-005
silLevel: 1
tags: [reg-unverified]
appliesWhen: Features::Protection::RapidShutdown
---

A signalling driver **shall** transmit the keep-alive and shutdown signal over the DC conductors.
