---
type: Requirement
id: REQ-GTI-SYS-009
name: "Anti-islanding"
status: approved
reqDomain: system
verificationMethod: test
derivedFrom:
  - REQ-GTI-STK-001
  - REQ-GTI-STK-003
breakdownAdr: ADR-GTI-REQ-001
derivedFromSafetyGoal: SG-GTI-001
silLevel: 2
---

When the grid is lost and the local load matches the generation, the inverter **shall** cease to energise within the island-clearing time of the profile (2 s in both regions) `[secondary, RM-09]`.
