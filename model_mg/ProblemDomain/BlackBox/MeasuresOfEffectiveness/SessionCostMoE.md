---
type: CalculationDef
name: SessionCostMoE
returnType: ScalarValues::Real
expression: "moe = capexPerStall / sessionsLifetime + energyCostPerSession"
custom_fields:
  mg_cell: B4
  mg_moe: true
  mg_moe_measures: ProblemDomain::WhiteBox::SystemRequirements::SessionCost
  mg_moe_unit: USD
  mg_moe_direction: minimize
  mg_moe_threshold: 4.0
  mg_moe_objective: 2.0
  mg_moe_weight: 0.20
---

**MoE — Cost per Session.** Effectiveness at the affordability need: amortised
capex per stall over lifetime sessions plus per-session energy cost (USD).
Minimised; above USD 4.00 is a knock-out.
