---
type: ThreatScenario
id: TS-SEC-102
name: Attacker replays captured torque frame (treated)
status: approved
attackFeasibility: high
attackVector: local
damageScenarios:
  - DS-SEC-101
riskTreatment: reduce
residualRisk: Low after message authentication code on torque frames
---

Same critical computed risk as TS-SEC-101, but carries a `riskTreatment` →
**no W031**.
