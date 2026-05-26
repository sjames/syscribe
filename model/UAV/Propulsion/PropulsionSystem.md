---
type: PartDef
name: PropulsionSystem
supertype: Parts::Part
isAbstract: true
features:
  - name: totalThrustN
    typedBy: ISQ::ForceValue
    isDerived: true
  - name: powerIn
    type: Port
    typedBy: Interfaces::PowerPortReceiverDef
    direction: in
    multiplicity: "1"
---

Abstract propulsion system definition. Concrete subtypes specify the rotor count and configuration (quad, hex, octo).
