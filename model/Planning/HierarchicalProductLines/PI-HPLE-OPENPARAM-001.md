---
type: PlanningItem
id: PI-HPLE-OPENPARAM-001
name: "Opt-in, --deny-gateable warning for an unresolved required parameter anywhere in a consolidated subtree"
status: todo
itemType: task
parent: PI-HPLE-001
achieves: [REQ-TRS-HPLE-004]
tags:
  - variability
  - multi-repo
  - validation
---

Compute the transitive closure of unbound `isRequired: true` parameters across an entire
consolidated subtree; report as a warning (never a hard error at an intermediate tier's own
isolated validation run), following the existing `W510`/`W511`/`W512`/`W023`/`W090` opt-in,
`--deny`-gateable posture.
