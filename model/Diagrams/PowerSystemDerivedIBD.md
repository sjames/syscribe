---
type: Diagram
name: PowerSystemDerivedIBD
diagramKind: IBD
subject: UAV::Power::PowerSystem
---

Internal block diagram **derived from the model** (`REQ-TRS-VIS-003`): the `PowerSystem` boundary
with its `mainPowerOut` port, the `battery` and `pdu` part usages with the ports their definitions
declare, the `PowerConnectionDef` connection and the binding to the boundary port — all read from
`UAV::Power::PowerSystem`'s own frontmatter. Compare `PowerSystemIBD`, the hand-listed manifest
form of the same view.
