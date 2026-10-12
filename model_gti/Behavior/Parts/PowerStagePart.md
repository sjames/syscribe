---
type: PartDef
name: PowerStagePart
exhibitsStates: [Behavior::Machines::PowerStage]
models: [System::Hardware::PowerStage]
fidelity: "drive enable/disable only; no electrical model"
abstracts: [dcIn, acOut, tempOut, REQ-GTI-SYS-037]   # no 10 us overcurrent trip
portMap:
  cmd: gate
  hw: hwDisable
features:
  - {name: cmd, type: Port, direction: in}
  - {name: hw, type: Port, direction: in}
---

Power stage gate drive.
