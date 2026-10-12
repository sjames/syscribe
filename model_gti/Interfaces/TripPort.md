---
type: PortDef
name: TripPort
features:
  - {name: trip, typedBy: Interfaces::Trip, direction: out}
  - {name: rearm, typedBy: Interfaces::Rearm, direction: out}
---

The hardware trip line, from the comparator to relay B and the gate-drive disable.
