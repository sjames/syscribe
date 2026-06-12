---
type: DamageScenario
id: DS-SEC-004
name: Damage scenario whose hazardRef resolves to a non-safety element
status: draft
damageSeverity: major
impactCategories:
  - safety
hazardRef: Engine
---

`hazardRef` resolves to a PartDef, not a HazardousEvent/SafetyGoal — must
produce E844.
