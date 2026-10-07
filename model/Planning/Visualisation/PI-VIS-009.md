---
type: PlanningItem
id: PI-VIS-009
name: "Phase 4 — Action diagram kind and generator with golden tests, demo diagram, writers and docs"
status: done
itemType: task
parent: PI-VIS-007
achieves: [REQ-TRS-VIS-019]
evidence:
  - path: repo:crates/syscribe-model/src/vis/derive/action.rs
  - path: repo:crates/syscribe-model/tests/vis_derive_behaviour.rs
  - path: repo:crates/syscribe-model/tests/vis_snapshots/derived/mission_action.json
  - path: repo:model/Diagrams/MissionExecutionDerivedAction.md
tags:
  - visualisation
---

`diagramKind: Action` in spec §8.16.1/8.16.2 and the new §8.16.8.8, `vis::derive::action`
(stereotyped steps with compartments, `IfAction` as decision/branches/merge, `LoopAction` as a
container, fork/join/decision/merge control nodes, successions and flows, initial/final only when
successions exist), the golden IR test and snapshot, the derived demo diagram of
`Behavior::MissionExecution`, the Mermaid `{{ }}`/`[[ ]]` mapping, the SVG bars and diamonds,
the browser views for the glyph kinds, the diagrams guide, and the qual mirror
`REQ`/`TC-TRS-VIS-019`.
