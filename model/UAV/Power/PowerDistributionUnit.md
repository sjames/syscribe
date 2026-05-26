---
type: PartDef
name: PowerDistributionUnit
supertype: Parts::Part
features:
  - name: powerIn
    type: Port
    typedBy: Interfaces::PowerPortReceiverDef
    direction: in
    multiplicity: "1"
  - name: powerOut
    type: Port
    typedBy: Interfaces::PowerPortDef
    direction: out
    multiplicity: "0..*"
---

Power distribution unit routing battery power to all UAV subsystems. Provides overcurrent protection per output rail.
