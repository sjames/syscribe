---
type: ThreatScenario
id: TS-SEC-103
name: Attacker spoofs sensor bus to suppress safety reaction (addressed)
status: approved
attackFeasibility: high
attackVector: local
damageScenarios:
  - DS-SEC-101
---

Same critical computed risk as TS-SEC-101, but addressed by CSG-SEC-101 (which
lists it in `threatScenarios`) → **no W031**.
