---
id: TS-ROOT-001
type: ThreatScenario
name: Attacker injects a forged actuator command
status: approved
attackFeasibility: high
attackVector: local
damageScenarios:
  - DS-ROOT-001
---

Declares `medium`, which matches the tree rolled up from its true root
(ATG-ROOT-002 = max(low, medium) = medium). Rolling up from the sub-gate
ATG-ROOT-001 instead would give `low` and raise a spurious W035.
