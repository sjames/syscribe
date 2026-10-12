---
type: PartDef
name: Monitor
models: [Arch::Manager]
fidelity: fast stage only
portMap: {out: trip}
features:
  - {name: samples, type: Port, direction: in}
  - {name: out, type: Port, direction: out}
  - {name: debug, type: Port, direction: out}
---

Executable model of the manager.
