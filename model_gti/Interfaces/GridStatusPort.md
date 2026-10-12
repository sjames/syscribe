---
type: PortDef
name: GridStatusPort
features:
  - {name: good, typedBy: Interfaces::GridGood, direction: out}
  - {name: lost, typedBy: Interfaces::GridLost, direction: out}
---

Grid qualification results from the protection function to the supervisor.
