---
type: PartDef
name: String Monitor
domain: software
appliesWhen: Features::Mppt::StringMonitoring
satisfies:
  - REQ-GTI-SYS-024
features:
  - {name: stringData, type: Port, direction: in}
  - {name: warning, type: Port, direction: out}
---

Compares string currents within a tracker and reports under-performance.
