---
type: PartDef
name: Motor
supertype: Parts::Part
metadata:
  - type: Metadata::DALAnnotation
    level: 3
    rationale: "Motor failure leads to degraded flight performance"
features:
  - name: kvRating
    typedBy: ScalarValues::Real
    isReadonly: true
  - name: maxPowerW
    typedBy: ScalarValues::Real
    unit: SI::W
    isReadonly: true
  - name: powerIn
    type: Port
    typedBy: Interfaces::PowerPortReceiverDef
    direction: in
---

Brushless DC motor driving a single rotor. The kV rating (RPM per volt) and maximum power draw are fixed hardware parameters.
