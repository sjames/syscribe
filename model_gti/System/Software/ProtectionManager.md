---
type: PartDef
name: Protection Manager
domain: software
silLevel: 2
satisfies:
  - REQ-GTI-SW-001
  - REQ-GTI-SW-002
  - REQ-GTI-SW-008
features:
  - {name: gridSamples, type: Port, direction: in}
  - {name: arcTrip, type: Port, direction: in}
  - {name: profile, type: Port, direction: in}
  - {name: trip, type: Port, direction: out}
  - {name: relayCmd, type: Port, direction: out}
---

Voltage/frequency trip table, anti-islanding perturbation and detection, relay command; see Behavior::GridMonitor.
