---
type: PlanningItem
id: PI-VIS-012
name: "Phase 4 — Allocation generator with golden tests, demo diagram, writers and docs"
status: done
itemType: task
parent: PI-VIS-007
achieves: [REQ-TRS-VIS-022]
evidence:
  - path: repo:crates/syscribe-model/src/vis/derive/allocation.rs
  - path: repo:crates/syscribe-model/tests/vis_derive_trace.rs
  - path: repo:crates/syscribe-model/tests/vis_snapshots/derived/alloc.json
  - path: repo:model/Diagrams/FunctionAllocationDerived.md
tags:
  - visualisation
---

`vis::derive::allocation`, golden IR test, derived demo diagram of the `Allocations` package, writer checks, qual mirror.

Landed as `vis::derive::allocation` (every pair form — an `Allocation` element's own ends, its
`features:` entries of `type: Allocation`, an `AllocationDef`'s `allocations:`, and
`allocatedTo:` on parts and actions — into a logical and a physical swimlane with one labelled
`«allocate»` edge per pair, dashed unresolved ends, filters on the end elements), the
integration test `vis_derive_trace.rs` with the golden `alloc.json`, the demo
`Diagrams::FunctionAllocationDerived`, spec §8.16.8.6 "Derived content",
`docs/format/diagrams.md`, and the qual mirror `TC-TRS-VIS-022`.
