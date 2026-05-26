---
type: ItemDef
name: ControlCommand
supertype: Items::Item
features:
  - name: commandType
    typedBy: Enumerations::FlightMode
  - name: targetAltitudeM
    typedBy: ScalarValues::Real
    unit: SI::m
---

A control command issued by the flight controller to the propulsion system or by the operator console to the UAV.
