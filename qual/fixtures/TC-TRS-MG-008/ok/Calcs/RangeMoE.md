---
type: CalculationDef
name: RangeMoE
custom_fields:
  mg_moe: true
  mg_moe_measures: REQ-MG-RANGE-001
  mg_moe_unit: km
  mg_moe_direction: maximize
  mg_moe_threshold: 10
  mg_moe_objective: 25
---

A well-formed MoE. The MoP BatteryCapacityMoP refines it, so this MoE should
report BatteryCapacityMoP under mopRefinedBy in show.
