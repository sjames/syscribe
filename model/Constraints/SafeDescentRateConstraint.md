---
type: ConstraintDef
name: SafeDescentRateConstraint
supertype: Constraints::Constraint
features:
  - name: descentRateMs
    typedBy: ScalarValues::Real
    direction: in
  - name: maxDescentRateMs
    typedBy: ScalarValues::Real
    direction: in
    value: "3.0"
    valueKind: default-bound
expression: "descentRateMs <= maxDescentRateMs"
---

Landing descent rate must not exceed 3 m/s to prevent structural damage on touchdown.
