---
type: Requirement
id: REQ-GTI-SYS-032
name: "DC arc-fault detection and interruption"
status: approved
reqDomain: system
verificationMethod: test
derivedFrom:
  - REQ-GTI-STK-003
breakdownAdr: ADR-GTI-REQ-001
derivedFromSafetyGoal: SG-GTI-002
silLevel: 2
tags: [reg-unverified]
appliesWhen: Features::Protection::Afci
---

The inverter **shall** detect a series DC arc and interrupt the affected input within the time defined by UL 1699B `[unverified, RM-14]`, and **shall** not restart automatically until the fault is cleared by a person.
