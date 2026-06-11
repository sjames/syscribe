---
type: ConstraintDef
name: PowerBudgetConstraint
parameters:
  - name: converterCount
    typedBy: ScalarValues::Real
  - name: converterPowerKw
    typedBy: ScalarValues::Real
expression: "converterCount * converterPowerKw >= 150.0"
custom_fields:
  mg_cell: W4
  mg_mop: true
  mg_mop_refines: ProblemDomain::BlackBox::MeasuresOfEffectiveness::DeliveredPowerMoE
  mg_mop_unit: kW
---

Parametric constraint: the installed converter power (module count times
per-module kW) must meet the 150 kW minimum delivery requirement.
