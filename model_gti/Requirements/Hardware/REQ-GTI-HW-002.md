---
type: Requirement
id: REQ-GTI-HW-002
name: "Series grid relays with weld detection"
status: approved
reqDomain: hardware
verificationMethod: test
derivedFrom:
  - REQ-GTI-SYS-009
breakdownAdr: ADR-GTI-REQ-001
derivedFromSafetyGoal: SG-GTI-001
silLevel: 2
---

Two series relays **shall** be driven by independent drivers, each with auxiliary contact feedback readable by the controller so that a welded contact is detected before every connection.
