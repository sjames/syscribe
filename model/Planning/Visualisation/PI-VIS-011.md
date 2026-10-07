---
type: PlanningItem
id: PI-VIS-011
name: "Phase 4 — Sequence generator with self-placement, golden tests, demo diagram, writers and docs"
status: done
itemType: task
parent: PI-VIS-007
achieves: [REQ-TRS-VIS-021]
evidence:
  - path: repo:crates/syscribe-model/src/vis/derive/sequence.rs
  - path: repo:crates/syscribe-model/tests/vis_derive_sequence.rs
  - path: repo:crates/syscribe-model/tests/vis_snapshots/derived/mission_seq.json
  - path: repo:model/Diagrams/MissionExecutionDerivedSeq.md
  - ref: TC-TRS-VIS-021
tags:
  - visualisation
---

`vis::derive::sequence` with pinned placement (lifelines at a fixed pitch, message rows, fragment
boxes, the subject's activation, horizontal message waypoints — every renderer draws it with
`fixed`, no ELK run), the golden IR test, the derived demo diagram of
`Behavior::MissionExecution`, the sequence kinds in the SVG writer and the sprotty client
(lifeline stems, actor figure, activation bar, fragment tab, messages along their row), the
`W080` manifest-only gate, spec §8.16.8.3 "Derived content", the docs entry and the qual mirror.
