---
type: ConstraintDef
name: SessionCostConstraint
parameters:
  - name: energyPricePerKwh
    typedBy: ScalarValues::Real
  - name: sessionEnergyKwh
    typedBy: ScalarValues::Real
expression: "energyPricePerKwh * sessionEnergyKwh <= 25.0"
custom_fields:
  mg_cell: W4
  mg_mop: true
  mg_mop_refines: ProblemDomain::BlackBox::MeasuresOfEffectiveness::SessionCostMoE
  mg_mop_unit: currency
---

White-box Measurement of Performance bounding the per-session cost, refining the
black-box **SessionCostMoE**. Keeps a typical charging session at or below the
target price point that makes the station competitive.
