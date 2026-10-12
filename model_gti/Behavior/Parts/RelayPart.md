---
type: PartDef
name: RelayPart
exhibitsStates: [Behavior::Machines::Relay]
models: [System::Hardware::AcOutput]
fidelity: "relay open/close and hardware trip; no weld detection"
abstracts: [acIn, gridOut, relayAux, acSense, REQ-GTI-HW-002]   # no weld detection
features:
  - {name: cmd, type: Port, direction: in, typedBy: Interfaces::RelayCmdPort, isConjugated: true}
  - {name: trip, type: Port, direction: in, typedBy: Interfaces::TripPort, isConjugated: true}
---

Grid relay.
