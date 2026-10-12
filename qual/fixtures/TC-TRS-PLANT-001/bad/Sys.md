---
type: PartDef
name: Sys
plantStep: "10 ms"
features:
  - {name: ctl, typedBy: CtlP}
  - {name: lag, typedBy: LagP}
bindingConnections:
  - {from: ctl.u, to: lag.u}
  - {from: lag.y, to: ctl.y}
---

The closed loop.
