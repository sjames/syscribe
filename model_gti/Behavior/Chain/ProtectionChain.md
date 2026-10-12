---
type: PartDef
name: ProtectionChain
plantStep: "50 ms"
features:
  - {name: grid, typedBy: Behavior::Parts::GridSourcePart, role: environment}
  - {name: gridZ, typedBy: Behavior::Parts::GridImpedancePart, role: environment}
  - {name: sensorSw, typedBy: Behavior::Parts::SoftwareSensorPart}
  - {name: sensorHw, typedBy: Behavior::Parts::SensorPart, portMap: {out: hwVoltage}}
  - {name: monitor, typedBy: Behavior::Parts::GridMonitorPart}
  - {name: perturb, typedBy: Behavior::Parts::PerturbationCheckPart}
  - {name: hwTrip, typedBy: Behavior::Parts::HwTripPart}
  - {name: supervisor, typedBy: System::Software::Supervisor}
  - {name: relayA, typedBy: Behavior::Parts::RelayPart}
  - {name: relayB, typedBy: Behavior::Parts::RelayPart}
  - {name: powerStage, typedBy: Behavior::Parts::PowerStagePart}
connections:
  - {from: grid.out, to: sensorSw.inp}
  - {from: grid.out, to: sensorHw.inp}
  - {from: grid.probe, to: sensorSw.inp}
  - {from: sensorSw.out, to: monitor.samples}
  - {from: sensorSw.out, to: perturb.samples}
  - {from: perturb.req, to: grid.req}
  - {from: perturb.trip, to: supervisor.trip}
  - {from: sensorHw.out, to: hwTrip.samples}
  - {from: monitor.out, to: supervisor.trip}
  - {from: supervisor.relayCmd, to: relayA.cmd}
  - {from: supervisor.relayCmd, to: relayB.cmd}
  - {from: supervisor.gridCtl, to: powerStage.cmd}
  - {from: hwTrip.out, to: relayB.trip}
  - {from: hwTrip.out, to: powerStage.hw}
bindingConnections:
  - {from: perturb.iInj, to: gridZ.iInj}
  - {from: grid.zGrid, to: gridZ.zGrid}
  - {from: grid.island, to: gridZ.island}
  - {from: gridZ.dv, to: grid.dv}
  - {from: powerStage.pwm, to: perturb.pwm}
---

The grid-disconnection chain of the inverter, US variant: two independent sensing chains (one for the STM32 monitor, one for the hardware trip), software monitor and supervisor on the STM32, an independent hardware trip that opens the second relay and disables the gate drive, two series relays, and the power stage. A perturbation-response check on the STM32 injects a small current while the bridge switches and measures how the grid answers, through the software sensing chain; the grid's answer comes from an in-process plant (`Behavior::Plants::GridImpedance`, exchanged with the machines every 50 ms, the `plantStep`, and integrated in steps of at most 10 ms), so the check sees a connected, weak or islanded grid as it would be. The limits and times come from the grid profile of the Configuration it is run with: US or India.
