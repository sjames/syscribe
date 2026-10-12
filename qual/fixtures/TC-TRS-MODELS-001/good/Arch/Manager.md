---
type: PartDef
name: Manager
domain: software
satisfies: [REQ-QM-001]
features:
  - {name: samples, type: Port, direction: in}
  - {name: trip, type: Port, direction: out}
  - {name: profile, type: Port, direction: in}
---

The protection manager.
