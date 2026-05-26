---
type: ConstraintDef
name: MassMarginConstraint
supertype: Constraints::Constraint
features:
  - name: totalMassKg
    typedBy: ScalarValues::Real
    direction: in
  - name: maxTakeoffMassKg
    typedBy: ScalarValues::Real
    direction: in
    value: "5.0"
    valueKind: default-bound
expression: "totalMassKg <= maxTakeoffMassKg"
---

MTOM constraint — total vehicle mass must not exceed 5 kg for sub-250g class waiver.
