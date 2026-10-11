---
type: Requirement
id: REQ-GTI-SW-002
name: "Anti-islanding detection"
status: approved
reqDomain: software
verificationMethod: test
derivedFrom:
  - REQ-GTI-SYS-009
breakdownAdr: ADR-GTI-REQ-001
derivedFromSafetyGoal: SG-GTI-001
silLevel: 2
---

The protection manager **shall** use active frequency or phase perturbation plus passive monitoring to detect an island and open the relays within the island-clearing time of the profile.
