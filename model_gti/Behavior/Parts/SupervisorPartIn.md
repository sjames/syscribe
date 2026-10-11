---
type: PartDef
name: SupervisorPartIn
exhibitsStates: [Behavior::Machines::SupervisorIn]
features:
  - {name: grid, type: Port, direction: in}
  - {name: relayCmd, type: Port, direction: out}
  - {name: drive, type: Port, direction: out}
---

Connection supervisor.
