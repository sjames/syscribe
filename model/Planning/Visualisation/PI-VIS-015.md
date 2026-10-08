---
type: PlanningItem
id: PI-VIS-015
name: "Place hand-listed Sequence diagrams and stop fixed layout on unpinned graphs"
status: done
itemType: bug
achieves: [REQ-TRS-VIS-025]
evidence:
  - path: repo:crates/syscribe-model/src/vis/derive/sequence.rs
  - path: repo:crates/syscribe-model/tests/vis_derive_sequence.rs
tags:
  - visualisation
---

`derive::sequence::place_manifest` called from `vis::build_graph`, the fixed-only-when-fully-pinned
guard in `vis::layout` and `vis::sprotty`'s layout options, golden test on the demo
`MissionExecutionSeq`, export and browser checks, qualification mirror.
