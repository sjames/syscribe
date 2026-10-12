---
type: PartDef
name: ProtectionChain
features:
  - {name: grid, typedBy: Behavior::Parts::GridSourcePart}
  - {name: sensorSw, typedBy: Behavior::Parts::SensorPart}
  - {name: sensorHw, typedBy: Behavior::Parts::SensorPart, portMap: {inp: acSig, out: hwVoltage}}
  - {name: monitor, typedBy: Behavior::Parts::GridMonitorPart}
  - {name: hwTrip, typedBy: Behavior::Parts::HwTripPart}
  - {name: supervisor, typedBy: Behavior::Parts::SupervisorPart}
  - {name: relayA, typedBy: Behavior::Parts::RelayPart}
  - {name: relayB, typedBy: Behavior::Parts::RelayPart}
  - {name: powerStage, typedBy: Behavior::Parts::PowerStagePart}
connections:
  - {from: grid.out, to: sensorSw.inp}
  - {from: grid.out, to: sensorHw.inp}
  - {from: sensorSw.out, to: monitor.samples}
  - {from: sensorHw.out, to: hwTrip.samples}
  - {from: monitor.out, to: supervisor.grid}
  - {from: supervisor.relayCmd, to: relayA.cmd}
  - {from: supervisor.relayCmd, to: relayB.cmd}
  - {from: supervisor.drive, to: powerStage.cmd}
  - {from: hwTrip.out, to: relayB.trip}
  - {from: hwTrip.out, to: powerStage.hw}
---

The grid-disconnection chain of the inverter, US variant: two independent sensing chains (one for the STM32 monitor, one for the hardware trip), software monitor and supervisor on the STM32, an independent hardware trip that opens the second relay and disables the gate drive, two series relays, and the power stage. The limits and times come from the grid profile of the Configuration it is run with: US or India.
