---
type: PortDef
name: DrivePort
features:
  - {name: enable, typedBy: Interfaces::Enable, direction: out}
  - {name: shutdown, typedBy: Interfaces::Shutdown, direction: out}
---

Power-stage enable and shutdown commands from the supervisor.
