---
type: PartDef
name: Rapid Shutdown Driver
domain: hardware
silLevel: 1
appliesWhen: Features::Protection::RapidShutdown
satisfies:
  - REQ-GTI-HW-005
features:
  - {name: cmd, type: Port, direction: in}
  - {name: initiator, type: Port, direction: in}
  - {name: dcLine, type: Port, direction: out}
---

Power-line signalling transmitter that keeps module-level shutdown devices alive and commands shutdown.
