---
type: PlanningItem
id: PI-VIS-003
name: "Phase 2 — ELK layout in the browser via sprotty-elk, pins, Pin all / Auto-layout / Save companion SVG, port-aware editing, shared style"
status: in_progress
itemType: feature
achieves: [REQ-TRS-VIS-006, REQ-TRS-VIS-007, REQ-TRS-VIS-008, REQ-TRS-VIS-011, REQ-TRS-VIS-012]
evidence:
  - path: repo:crates/syscribe-server/frontend/src/layout.ts
  - path: repo:crates/syscribe-server/tests/layout_routes.rs
  - path: repo:crates/syscribe-model/src/vis/style.rs
  - path: repo:crates/syscribe-server/frontend/test/elk-layout.test.mjs
tags:
  - visualisation
---

Vendor `sprotty-elk` and `elkjs`, enable client bounds measurement, bind the ELK layout
engine, implement pin semantics and the three new actions, make connect port-aware, and move
the visual language into `vis::style`. Design: `docs/design/visualisation.md` §6, §7.

Landed in `cacff379` (backend: `vis::style`, style carried in the sprotty graph, `PATCH` `null`,
`DELETE /api/diagrams/layout`, `PUT /api/diagrams/svg`) and `d2e72234` (client: ELK via
`sprotty-elk`/`elkjs`, measured labels, pins through interactive layering and `fixed` mode,
the three buttons, port-aware connect, `npm test`). Four of the five requirements are verified
(`TC-TRS-VIS-006/007/011/012`). `REQ-TRS-VIS-008`'s connect rules (`connect-rules.ts`) have no
automated test yet, so `TC-TRS-VIS-008` is `draft` and this item stays `in_progress` rather than
`done` — marking it `done` would raise `W310`. The one remaining step is a Node test under
`frontend/test/` driving `resolveConnectEnds`, `portChain` and `isDerivedDiagram`; when it lands,
activate `TC-TRS-VIS-008` and close this item.
