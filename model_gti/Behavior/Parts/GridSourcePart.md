---
type: PartDef
name: GridSourcePart
exhibitsStates: [Behavior::Machines::GridSource]
features:
  - {name: out, type: Port, direction: out, typedBy: Interfaces::SamplePort}
  - {name: probe, type: Port, direction: out, typedBy: Interfaces::SamplePort}
  - {name: req, type: Port, direction: in, typedBy: Interfaces::ProbePort, isConjugated: true}
---

Test environment grid.
