---
type: PlanningItem
id: PI-BT-001
name: "Implement binding time as a staged-configuration axis"
status: todo
itemType: feature
achieves: [REQ-TRS-BT-000]
tags:
  - variability
  - binding-time
---

Top-level item for `ADR-SYS-BT-001` / `REQ-TRS-BT-000`. Four delivery phases, in order: (1) vocabulary and windows, with the single `BindingTime` type; (2) stages and provenance; (3) residual variability and the parameter file; (4) projection, verification and UI. Migration, documentation and the worked example (`PI-BT-MIG-001`) close it out. Each phase must leave `validate` clean on `model/` and the examples (`REQ-TRS-BT-038`).
