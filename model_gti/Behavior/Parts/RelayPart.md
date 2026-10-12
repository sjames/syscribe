---
type: PartDef
name: RelayPart
exhibitsStates: [Behavior::Machines::Relay]
models: [System::Hardware::AcOutput]
fidelity: "relay open/close and hardware trip; no weld detection"
abstracts: [acIn, gridOut, relayAux, acSense, REQ-GTI-HW-002]   # no weld detection
portMap:
  cmd: relayDrive
  trip: relayTrip
features:
  - {name: cmd, type: Port, direction: in}
  - {name: trip, type: Port, direction: in}
---

Grid relay.
