---
type: PlanningItem
id: PI-VIS-008
name: "Phase 4 — StateMachine generator with golden tests, demo diagram, writers and docs"
status: done
itemType: task
parent: PI-VIS-007
achieves: [REQ-TRS-VIS-018]
evidence:
  - path: repo:crates/syscribe-model/src/vis/derive/state.rs
  - path: repo:crates/syscribe-model/tests/vis_derive_behaviour.rs
  - path: repo:crates/syscribe-model/tests/vis_snapshots/derived/flight_sm.json
  - path: repo:model/Diagrams/FlightStatesDerivedSM.md
tags:
  - visualisation
---

`vis::derive::state` (states with entry/do/exit compartments, initial/final pseudostates per
region, transitions from both placements and the deprecated aliases labelled
`accept [guard] / effect`, a typed substate as a one-level container), the golden IR test and
snapshot, the derived demo diagram of `Behavior::FlightStates`, the Mermaid composite-state and
PlantUML `[*]`/description writers, glyph sizing and ELK layer constraints for the pseudostates,
spec §8.16.8.4 "Derived content", the diagrams guide, and the qual mirror
`REQ`/`TC-TRS-VIS-018`.
