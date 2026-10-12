---
type: PartDef
name: GridMonitorPart
exhibitsStates: [Behavior::Machines::GridMonitor]
models: [System::Software::ProtectionManager]
fidelity: "fast-stage voltage/frequency window only"
abstracts: [arcTrip, profile, relayCmd]
portMap:
  samples: gridSamples
  out: trip
features:
  - {name: samples, type: Port, direction: in}
  - {name: out, type: Port, direction: out}
---

Software grid monitor.
