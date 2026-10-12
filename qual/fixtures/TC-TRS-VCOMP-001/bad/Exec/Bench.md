---
type: PartDef
name: Bench
features:
  - {name: stim, typedBy: Exec::Stim, role: environment, portMap: {out: out}}
  - {name: mon, typedBy: Exec::Monitor}
  - {name: rel, typedBy: Arch::Relay}
  - {name: spare, typedBy: Exec::Monitor, role: enviroment}
connections:
  - {from: stim.out, to: mon.samples}
  - {from: mon.out, to: rel.cmd}
---

The verification composite.
