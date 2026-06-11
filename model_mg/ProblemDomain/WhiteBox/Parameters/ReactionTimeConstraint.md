---
type: ConstraintDef
name: ReactionTimeConstraint
parameters:
  - name: detectionMs
    typedBy: ScalarValues::Real
  - name: contactorTripMs
    typedBy: ScalarValues::Real
expression: "detectionMs + contactorTripMs <= 100.0"
custom_fields:
  mg_cell: W4
  mg_mop: true
  mg_mop_refines: ProblemDomain::BlackBox::MeasuresOfEffectiveness::SafetyReactionMoE
  mg_mop_unit: ms
---

Parametric constraint: total fault-detection plus contactor-trip time must stay
within the 100 ms safety reaction budget.
