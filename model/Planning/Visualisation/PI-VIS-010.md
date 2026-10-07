---
type: PlanningItem
id: PI-VIS-010
name: "Phase 4 — Requirement generator with golden tests, demo diagram, writers and docs"
status: done
itemType: task
parent: PI-VIS-007
achieves: [REQ-TRS-VIS-020]
evidence:
  - path: repo:crates/syscribe-model/src/vis/derive/requirement.rs
  - path: repo:crates/syscribe-model/tests/vis_derive_trace.rs
  - path: repo:crates/syscribe-model/tests/vis_snapshots/derived/reqs.json
  - path: repo:model/Diagrams/RequirementsDerived.md
tags:
  - visualisation
---

`vis::derive::requirement`, golden IR test, derived demo diagram of the `Requirements` package, writer checks, qual mirror.

Landed as `vis::derive::requirement` (requirement nodes with id/status compartments, derive,
refine, containment, satisfy and verify edges, context nodes with their real type's stereotype,
filters by qualified name, stable id or short name), the integration test
`vis_derive_trace.rs` with the golden `reqs.json`, the PlantUML requirement writer emitting the
compartment lines as the class body, the demo `Diagrams::RequirementsDerived` (narrowed with
`include:` to the UAV trees), spec §8.16.8.5 "Derived content", `docs/format/diagrams.md`, and
the qual mirror `TC-TRS-VIS-020`.
