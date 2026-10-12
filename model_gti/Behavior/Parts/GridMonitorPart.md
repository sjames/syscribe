---
type: PartDef
name: GridMonitorPart
exhibitsStates: [Behavior::Machines::GridMonitor]
models: [System::Software::ProtectionManager]
fidelity: "fast-stage voltage/frequency window only"
abstracts: [arcTrip, profile, relayCmd]
features:
  - {name: samples, type: Port, direction: in, typedBy: Interfaces::SamplePort, isConjugated: true}
  - {name: out, type: Port, direction: out, typedBy: Interfaces::GridStatusPort}
---

Software grid monitor.
