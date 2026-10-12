---
type: PortDef
name: RelayCmdPort
features:
  - {name: close, typedBy: Interfaces::Close, direction: out}
  - {name: shutdown, typedBy: Interfaces::Shutdown, direction: out}
---

Relay commands from the supervisor.
