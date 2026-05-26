---
type: PartDef
name: IMU
supertype: Parts::Part
features:
  - name: rollRateDegS
    typedBy: ScalarValues::Real
    isReadonly: true
    isDerived: true
  - name: pitchRateDegS
    typedBy: ScalarValues::Real
    isReadonly: true
    isDerived: true
  - name: yawRateDegS
    typedBy: ScalarValues::Real
    isReadonly: true
    isDerived: true
  - name: accelXMs2
    typedBy: ScalarValues::Real
    isReadonly: true
    isDerived: true
  - name: powerIn
    type: Port
    typedBy: Interfaces::PowerPortReceiverDef
    direction: in
---

Inertial Measurement Unit providing angular rates and linear accelerations to the flight controller at 1 kHz. All sensor outputs are read-only derived values.
