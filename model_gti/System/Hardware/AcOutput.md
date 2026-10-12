---
type: PartDef
name: AC Output
domain: hardware
silLevel: 2
satisfies:
  - REQ-GTI-SYS-038
  - REQ-GTI-HW-002
  - REQ-GTI-HW-009
features:
  - {name: acIn, type: Port, direction: in}
  - {name: gridOut, type: Port, direction: out}
  - {name: relayDrive, type: Port, direction: in, typedBy: Interfaces::RelayCmdPort, isConjugated: true}
  - {name: relayTrip, type: Port, direction: in, typedBy: Interfaces::TripPort, isConjugated: true}
  - {name: relayAux, type: Port, direction: out}
  - {name: acSense, type: Port, direction: out}
---

LCL filter, two series grid relays with independent drivers and auxiliary-contact feedback (weld detection), type-II AC surge protection.
