---
type: PartDef
name: Bench
features:
  - {name: stim, typedBy: Exec::Stim, role: environment}
  - {name: mon, typedBy: Exec::Monitor}
  - {name: rel, typedBy: Arch::Relay}
connections:
  - {from: stim.out, to: mon.samples}
  - {from: mon.out, to: rel.cmd}
---

The verification composite.
