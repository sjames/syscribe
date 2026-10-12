---
type: PartDef
name: PowerStagePart
exhibitsStates: [Behavior::Machines::PowerStage]
models: [System::Hardware::PowerStage]
fidelity: "drive enable/disable only; no electrical model"
abstracts: [dcIn, acOut, tempOut, REQ-GTI-SYS-037]   # no 10 us overcurrent trip
features:
  - {name: cmd, type: Port, direction: in, typedBy: Interfaces::DrivePort, isConjugated: true}
  - {name: hw, type: Port, direction: in, typedBy: Interfaces::TripPort, isConjugated: true}
---

Power stage gate drive.
