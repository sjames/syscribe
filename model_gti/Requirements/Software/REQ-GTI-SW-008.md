---
type: Requirement
id: REQ-GTI-SW-008
name: "Perturbation-response check"
status: approved
reqDomain: software
verificationMethod: test
derivedFrom:
  - REQ-GTI-SYS-045
breakdownAdr: ADR-GTI-REQ-001
derivedFromSafetyGoal: SG-GTI-001
silLevel: 2
---

The protection manager **shall** apply a small, bounded perturbation (reactive-current or phase step, at most 1 % of rated current) at least once per second and compare the measured change in grid voltage and frequency with the change expected from the last estimated grid impedance. When the response is missing or deviates beyond the tolerance of the grid profile `[unverified]`, or the impedance estimate rises above the profile limit, it **shall** treat the grid as lost and open the relays within the island-clearing time. The check **shall** run in every state in which the bridge is switching.
