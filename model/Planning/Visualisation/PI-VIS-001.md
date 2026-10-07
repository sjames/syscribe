---
type: PlanningItem
id: PI-VIS-001
name: "Phase 0 — Diagram IR, one manifest parser, E405/W416, nested sprotty model, legacy renderers and CLI diagram toolkit removed"
status: todo
itemType: feature
achieves: [REQ-TRS-VIS-001, REQ-TRS-VIS-002, REQ-TRS-VIS-013]
tags:
  - visualisation
---

Clear the ground: `syscribe_model::vis::{ir, manifest}`, `E405`/`W416` catalogued, the
diagram-model endpoint emitting nesting, `sub_mapping` no longer silently replacing a scalar,
and the deletions listed in `REQ-TRS-VIS-013`. Design: `docs/design/visualisation.md` §5, §9,
§11.
