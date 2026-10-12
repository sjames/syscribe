---
type: PartDef
name: HwTripPart
exhibitsStates: [Behavior::Machines::HwTrip]
models: [System::Hardware::ControlBoard]
fidelity: "hardware overvoltage comparator only"
abstracts: [adcIn, arcIn, auxIn, pwmOut, relayCmd, rsdCmd, comms]
portMap:
  samples: hwVin
  out: [hwTripGate, hwTripRelay]   # one comparator output drives both trip lines
features:
  - {name: samples, type: Port, direction: in}
  - {name: out, type: Port, direction: out}
---

Hardware trip path.
