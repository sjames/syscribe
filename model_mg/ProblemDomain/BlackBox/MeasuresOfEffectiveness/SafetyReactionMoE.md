---
type: CalculationDef
name: SafetyReactionMoE
returnType: ScalarValues::Real
expression: "moe = detectionMs + contactorTripMs"
custom_fields:
  mg_cell: B4
  mg_moe: true
  mg_moe_measures: ProblemDomain::WhiteBox::SystemRequirements::SafetyIsolation
  mg_moe_unit: ms
  mg_moe_direction: minimize
  mg_moe_threshold: 100
  mg_moe_objective: 30
  mg_moe_weight: 0.20
---

**MoE — Fault Reaction Time.** Effectiveness at the safety need: total time to
de-energise = fault-detection time + contactor trip time (ms). Minimised; above
100 ms is a knock-out.
