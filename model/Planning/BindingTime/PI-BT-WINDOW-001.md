---
type: PlanningItem
id: PI-BT-WINDOW-001
name: "Windows: bindingTime/latestBindingTime on features and parameters, ordering checks"
status: todo
itemType: task
parent: PI-BT-PHASE1-001
achieves: [REQ-TRS-BT-006, REQ-TRS-BT-007, REQ-TRS-BT-008, REQ-TRS-BT-009, REQ-TRS-BT-010]
tags:
  - variability
  - binding-time
---

Add `latestBindingTime:` and feature-level windows; generalise `E229`/`E230` to windows; run ordering under `validate`; keep `TC-TRS-PARAM-004` green. Verified by `TC-TRS-BT-006..010`.
