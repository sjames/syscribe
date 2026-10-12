---
type: PartDef
name: Board
domain: software
features:
  - {name: sensor, typedBy: Arch::Sensor}
  - {name: mgr, typedBy: Arch::Manager}
  - {name: relay, typedBy: Arch::Relay}
connections:
  - {from: mgr.trip, to: relay.cmd}
---

The software board.
