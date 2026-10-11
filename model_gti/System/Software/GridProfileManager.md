---
type: PartDef
name: Grid Profile Manager
domain: software
satisfies:
  - REQ-GTI-SYS-060
features:
  - {name: write, type: Port, direction: in}
  - {name: profile, type: Port, direction: out}
---

Holds the active grid profile (limits and timings per region), loaded and locked at commissioning.
