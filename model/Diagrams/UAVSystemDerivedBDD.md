---
type: Diagram
name: UAVSystemDerivedBDD
diagramKind: BDD
subject: UAV
---

Block definition diagram **derived from the model** (`REQ-TRS-VIS-003`): every definition that is
a direct member of the `UAV` package, with its attribute and port compartments, inheritance from
`supertype:` and composition from part usages. The subsystem definitions that the members compose
(in other packages) are pulled in as *external* blocks one level deep (`REQ-TRS-BDDX-001`; `depth: 0`
restores members only). No `shapes:` are listed — the content follows the
model, and the browser lays it out.
