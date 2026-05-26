---
type: PartDef
name: RotorAssembly
supertype: Parts::Part
features:
  - name: motor
    typedBy: UAV::Propulsion::Motor
    multiplicity: "1"
  - name: diameterM
    typedBy: ScalarValues::Real
    unit: SI::m
    isReadonly: true
  - name: powerIn
    type: Port
    typedBy: Interfaces::PowerPortReceiverDef
    direction: in
---

A single rotor assembly consisting of a motor, propeller blade, and mounting hardware. Receives electrical power and generates thrust.
