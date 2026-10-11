---
type: PartDef
name: Insulation Monitor
domain: software
silLevel: 2
satisfies:
  - REQ-GTI-SW-003
features:
  - {name: measure, type: Port, direction: in}
  - {name: result, type: Port, direction: out}
---

Runs the insulation-resistance measurement and the continuous residual-current check.
