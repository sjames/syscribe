---
type: PartDef
name: Relay
domain: software
satisfies: [REQ-VC-002]
exhibitsStates: [Exec::RelayM]
features:
  - {name: cmd, type: Port, direction: in}
---

The relay driver; carries its own machine.
