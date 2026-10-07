---
type: PlanningItem
id: PI-VIS-003
name: "Phase 2 — ELK layout in the browser via sprotty-elk, pins, Pin all / Auto-layout / Save companion SVG, port-aware editing, shared style"
status: todo
itemType: feature
achieves: [REQ-TRS-VIS-006, REQ-TRS-VIS-007, REQ-TRS-VIS-008, REQ-TRS-VIS-011, REQ-TRS-VIS-012]
tags:
  - visualisation
---

Vendor `sprotty-elk` and `elkjs`, enable client bounds measurement, bind the ELK layout
engine, implement pin semantics and the three new actions, make connect port-aware, and move
the visual language into `vis::style`. Design: `docs/design/visualisation.md` §6, §7.
