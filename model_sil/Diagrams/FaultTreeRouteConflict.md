---
type: Diagram
name: FaultTreeRouteConflict
diagramKind: FaultTree
subject: Safety::FTA::FT-SIL-001
---

Fault tree for the safety goal `SG-SIL-001` (conflicting route set without detection) **derived from the
model**: the OR gate over the diverse-software AND gate and the 2oo2 comparison hardware, with the
`fault-tree analyze` overlay — the 2oo2 comparator is the single point of failure, the two diverse
software channels form a dual-point cut set, and the root carries the top-event probability.
