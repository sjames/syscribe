---
type: Requirement
id: REQ-GTI-SYS-010
name: "Reconnection after a trip"
status: approved
reqDomain: system
verificationMethod: test
derivedFrom:
  - REQ-GTI-STK-001
breakdownAdr: ADR-GTI-REQ-001
tags: [reg-unverified]
---

After a trip the inverter **shall** reconnect only after grid voltage and frequency have stayed inside the reconnection window continuously for the reconnect delay of the profile (US 300 s, India 60 s) `[unverified, RM-07, RM-08]`, and **shall** ramp power up rather than step.
