---
type: PartDef
name: Monitor
models: [Arch::Manager]
portMap: {out: trip}
exhibitsStates: [Exec::MonM]
features:
  - {name: samples, type: Port, direction: in}
  - {name: out, type: Port, direction: out}
---

Executable model of the manager.
