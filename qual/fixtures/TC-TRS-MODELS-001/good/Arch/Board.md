---
type: PartDef
name: Board
domain: software
features:
  - {name: mgr, typedBy: Arch::Manager}
  - {name: relay, typedBy: Arch::Relay}
connections:
  - {from: mgr.trip, to: relay.cmd}
---

The software composite.
