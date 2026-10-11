---
type: Requirement
id: REQ-GTI-SYS-025
name: "Never exceed the 1000 V input limit"
status: approved
reqDomain: system
verificationMethod: test
derivedFrom:
  - REQ-GTI-STK-003
  - REQ-GTI-STK-002
breakdownAdr: ADR-GTI-REQ-001
---

The inverter **shall** not start or connect a tracker whose open-circuit voltage exceeds 1000 V, and **shall** survive 1100 V without damage.
