---
id: AT-BAD-001
type: AttackTree
name: Attack tree with a threatRef that does not resolve to a ThreatScenario
threatRef: Ecu
status: approved
---

The `threatRef` points at a Part rather than a ThreatScenario, so validation must
emit E917.
