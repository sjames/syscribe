---
type: PlanningItem
id: PI-VIS-005
name: "Phase 4 — test coverage at every visualisation layer and the qualification mirror"
status: done
itemType: feature
achieves: [REQ-TRS-VIS-014]
evidence:
  - path: repo:crates/syscribe-model/tests/vis_manifest_validation.rs
  - path: repo:crates/syscribe-model/tests/vis_derive.rs
  - path: repo:crates/syscribe-model/tests/vis_writers.rs
  - path: repo:crates/syscribe-model/tests/vis_layout.rs
  - path: repo:crates/syscribe-server/tests/diagram_model.rs
  - path: repo:crates/syscribe-server/frontend/test/elk-layout.test.mjs
tags:
  - visualisation
---

Runs alongside every other phase; listed separately so it is never deferred. Golden IR
fixtures, writer snapshots, Axum endpoint tests, the Node-side `elkjs` layout check, and
`qual/` mirrors. Design: `docs/design/visualisation.md` §10.
