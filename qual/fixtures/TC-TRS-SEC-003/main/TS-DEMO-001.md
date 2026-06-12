---
id: TS-DEMO-001
type: ThreatScenario
name: Attacker injects a forged torque-request frame onto the powertrain bus
status: approved
attackFeasibility: high
attackVector: local
damageScenarios:
  - DS-DEMO-001
---

The threat scenario that AT-DEMO-001 substantiates. Its declared
`attackFeasibility: high` deliberately disagrees with the tree's computed
feasibility (medium) so that the reconciliation warning W035 fires.
