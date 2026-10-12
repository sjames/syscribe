---
type: PartDef
name: Monitor
models: [Arch::Manager]
fidelity: fast stage only
abstracts: [profile]
portMap: {out: trip}
features:
  - {name: samples, type: Port, direction: in}
  - {name: out, type: Port, direction: out}
---

Executable model of the manager.
