---
type: Requirement
id: REQ-TRS-PIACH-001
name: "PlanningItem.achieves accepts non-Requirement outcomes"
status: draft
reqDomain: software
reqClass: system
tags:
  - planning
---

A `PlanningItem` shall be able to achieve work products that are not requirements (a HARA, TARA, decision, safety case or baseline), with a completion check per target (GH #240).

## Behavior

- `achieves:` may name a native `Requirement` or a `SafetyGoal`, `CybersecurityGoal`, `ADR`, `Argument`, `TestPlan` or `Baseline`. Anything else is `E715` and the message lists the accepted types.
- A `done` item achieving a non-Requirement target whose own status is still `draft`, `review` or `proposed` raises `W315` (the work product has not been completed). A Requirement target keeps the existing `W310` verification bar.
- `syscribe set <PI> achieves.add <ref>` accepts the same targets.
- `E713` (a top-level item needs `achieves:`) and `E714` (unresolved) are unchanged.
