---
type: PlanningItem
id: PI-BT-PROJ-001
name: "Projection retains elements bound after the projected stage"
status: todo
itemType: task
parent: PI-BT-PHASE4-001
achieves: [REQ-TRS-BT-031, REQ-TRS-BT-032, REQ-TRS-BT-033]
tags:
  - variability
  - binding-time
---

Retain and mark variant-retained any element whose `appliesWhen:` depends on a later-bound item; report dependents that rely on its absence; keep unstaged projection byte-identical. Verified by `TC-TRS-BT-031..033`.
