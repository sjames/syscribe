---
type: Requirement
id: REQ-GTI-SYS-007
name: "Trip on voltage or frequency outside the profile limits"
status: approved
reqDomain: system
verificationMethod: test
derivedFrom:
  - REQ-GTI-STK-001
breakdownAdr: ADR-GTI-REQ-001
tags: [reg-unverified]
---

The inverter **shall** cease to energise and open its grid relays when grid voltage or frequency stays outside the limits of the active grid profile for longer than the clearing time stated by the profile `[unverified, RM-03..RM-06]`.
