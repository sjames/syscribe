---
type: PartDef
name: SupervisorPart
exhibitsStates: [Behavior::Machines::Supervisor]
models: [System::Software::Supervisor]
fidelity: "trip-to-disconnect path only"
abstracts: [profile, insulation, selfTestFail, cmdIn, mpptCtl, event]
features:
  - {name: grid, type: Port, direction: in, typedBy: Interfaces::GridStatusPort, isConjugated: true}
  - {name: relayCmd, type: Port, direction: out, typedBy: Interfaces::RelayCmdPort}
  - {name: drive, type: Port, direction: out, typedBy: Interfaces::DrivePort}
---

Connection supervisor.
