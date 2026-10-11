---
type: PartDef
name: SupervisorPartUs
exhibitsStates: [Behavior::Machines::SupervisorUs]
features:
  - {name: grid, type: Port, direction: in}
  - {name: relayCmd, type: Port, direction: out}
  - {name: drive, type: Port, direction: out}
---

Connection supervisor.
