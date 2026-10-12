---
type: PartDef
name: SupervisorPart
exhibitsStates: [Behavior::Machines::Supervisor]
models: [System::Software::Supervisor]
fidelity: "trip-to-disconnect path only"
abstracts: [profile, insulation, selfTestFail, cmdIn, mpptCtl, event]
portMap:
  grid: trip
  drive: gridCtl
features:
  - {name: grid, type: Port, direction: in}
  - {name: relayCmd, type: Port, direction: out}
  - {name: drive, type: Port, direction: out}
---

Connection supervisor.
