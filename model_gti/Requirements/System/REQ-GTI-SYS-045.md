---
type: Requirement
id: REQ-GTI-SYS-045
name: "Defence against a common cause that blinds both voltage-sensing chains"
status: approved
reqDomain: system
verificationMethod: test
derivedFrom:
  - REQ-GTI-STK-003
breakdownAdr: ADR-GTI-REQ-001
derivedFromSafetyGoal: SG-GTI-001
silLevel: 2
---

If both grid-voltage sensing chains report a wrong but plausible value at the same time (for example a wiring error at the grid connection point), the inverter **shall** still detect that the grid is not following its output, and **shall** cease to energise within the island-clearing time of the profile. The detection **shall** rely on how the measured grid responds to the inverter's own action, not on the absolute value of the voltage. Rationale: without this detection the sensing common cause FTE-GTI-009 of FT-GTI-001 is a single cut set (ADR-GTI-SAFE-001); with it, FTE-GTI-009 is a cut set of order 2 together with the failure of the check (FTE-GTI-010).
