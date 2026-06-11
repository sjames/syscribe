---
type: ConstraintDef
name: AvailabilityConstraint
parameters:
  - name: baseAvailability
    typedBy: ScalarValues::Real
  - name: redundancySpare
    typedBy: ScalarValues::Real
expression: "baseAvailability + redundancySpare * 0.005 >= 0.99"
custom_fields:
  mg_cell: W4
  mg_mop: true
  mg_mop_refines: ProblemDomain::BlackBox::MeasuresOfEffectiveness::AvailabilityMoE
  mg_mop_unit: fraction
---

Parametric constraint: base availability uplifted by spare converter modules
must meet the 99.0% availability target.
