---
type: ConstraintDef
name: PropulsionThrustConstraint
supertype: Constraints::Constraint
features:
  - name: thrustToWeightRatio
    typedBy: ScalarValues::Real
    direction: in
  - name: minThrustToWeight
    typedBy: ScalarValues::Real
    direction: in
    value: "2.0"
    valueKind: default-bound
expression: "thrustToWeightRatio >= minThrustToWeight"
---

Propulsion system must provide thrust-to-weight ratio of at least 2:1 for adequate maneuverability.
