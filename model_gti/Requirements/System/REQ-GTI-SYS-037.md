---
type: Requirement
id: REQ-GTI-SYS-037
name: "Independent hardware overcurrent trip"
status: approved
reqDomain: system
verificationMethod: test
derivedFrom:
  - REQ-GTI-STK-003
breakdownAdr: ADR-GTI-REQ-001
---

A hardware comparator path that does not depend on the main software loop **shall** turn off the switching stage within 10 µs of AC or bridge current exceeding twice rated peak.
