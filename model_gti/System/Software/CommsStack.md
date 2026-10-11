---
type: PartDef
name: Comms Stack
domain: software
satisfies:
  - REQ-GTI-SYS-053
  - REQ-GTI-SYS-050
  - REQ-GTI-SYS-051
features:
  - {name: fieldbus, type: Port, direction: inout}
  - {name: command, type: Port, direction: out}
  - {name: profileWrite, type: Port, direction: out}
  - {name: status, type: Port, direction: in}
  - {name: image, type: Port, direction: out}
---

SunSpec Modbus, IEEE 2030.5 client and authenticated command handling.
