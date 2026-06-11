---
type: CalculationDef
name: SpeedMoE
expression: "out = speed"
custom_fields:
  mg_moe: true
  mg_moe_measures: REQ-MG-SPEED-001
  mg_moe_unit: kph
  mg_moe_direction: maximize
  mg_moe_threshold: 0
  mg_moe_objective: 100
  mg_moe_weight: 1.0
---
Speed MoE; its expression variable `speed` is resolved from parameterBindings.
