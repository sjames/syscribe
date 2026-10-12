---
type: PartDef
name: Chain
features:
  - {name: mon, typedBy: Exec::Monitor}
  - {name: rel, typedBy: Exec::RelayM}
connections:
  - {from: mon.out, to: rel.cmd}
---

The executable chain.
