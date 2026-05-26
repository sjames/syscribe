---
type: ConstraintDef
name: BatteryEnergyConstraint
supertype: Constraints::Constraint
features:
  - name: capacityWh
    typedBy: ScalarValues::Real
    direction: in
  - name: nominalVoltageV
    typedBy: ScalarValues::Real
    direction: in
  - name: minCapacityWh
    typedBy: ScalarValues::Real
    direction: in
    value: "50.0"
    valueKind: default-bound
expression: "capacityWh >= minCapacityWh and nominalVoltageV >= 14.8"
---

Ensures battery meets minimum energy and voltage requirements for safe flight.
