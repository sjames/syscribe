---
type: CalculationDef
name: DeliveredPowerMoE
returnType: ScalarValues::Real
expression: "moe = converterCount * converterPowerKw"
custom_fields:
  mg_cell: B4
  mg_moe: true
  mg_moe_measures: ProblemDomain::WhiteBox::SystemRequirements::ChargePower
  mg_moe_unit: kW
  mg_moe_direction: maximize
  mg_moe_threshold: 150
  mg_moe_objective: 360
  mg_moe_weight: 0.35
---

**MoE — Peak Delivered Power.** Effectiveness of a configuration at meeting the
fast-turnaround need, computed as the installed power: number of converter
modules times per-module power (kW). Maximised; below 150 kW is a knock-out.
