---
type: PartDef
name: SupervisorPart
exhibitsStates: [Behavior::Machines::Supervisor]
features:
  - {name: grid, type: Port, direction: in}
  - {name: relayCmd, type: Port, direction: out}
  - {name: drive, type: Port, direction: out}
---

Connection supervisor.
