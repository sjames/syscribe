---
type: PartDef
name: BatteryPack
supertype: Parts::Part
domain: hardware
satisfies:
  - REQ-UAV-ENDUR-001
features:
  - name: capacityWh
    typedBy: ScalarValues::Real
    unit: SI::J
    isConstant: true
    isReadonly: true
    value: "133.56"
  - name: nominalVoltageV
    typedBy: ScalarValues::Real
    unit: SI::V
    isConstant: true
    isReadonly: true
    value: "22.2"
  - name: massKg
    typedBy: ScalarValues::Real
    unit: SI::kg
    isReadonly: true
    isNonunique: false
    isOrdered: false
  - name: powerOut
    type: Port
    typedBy: Interfaces::PowerPortDef
    direction: out
---

6S LiPo battery pack. Capacity is fixed at manufacturing time. Nominal voltage is 22.2 V (3.7 V/cell nominal). Provides main power bus supply.
