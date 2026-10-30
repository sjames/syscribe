---
type: PlanningItem
id: PI-BT-PARAMFILE-001
name: "Late-bound parameter file: params export, schema, check, staleness"
status: todo
itemType: task
parent: PI-BT-PHASE3-001
achieves: [REQ-TRS-BT-023, REQ-TRS-BT-024, REQ-TRS-BT-025, REQ-TRS-BT-026, REQ-TRS-BT-027, REQ-TRS-BT-028, REQ-TRS-BT-029, REQ-TRS-BT-030]
tags:
  - variability
  - binding-time
---

Add `params export` (YAML keyed by `FEAT-*` id, deterministic, strict by default), `fallback:`, `consumedBy:`, `--schema` and `params check` with the projection-hash staleness warning. Verified by `TC-TRS-BT-023..030`.
