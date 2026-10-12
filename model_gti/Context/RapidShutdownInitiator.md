---
type: PartDef
name: Rapid Shutdown Initiator
appliesWhen: Features::Protection::RapidShutdown
features:
  - {name: cmd, type: Port, direction: out, flowKind: command}
---

The NEC 690.12 initiation device (a switch at the service entrance) that commands rapid shutdown.
