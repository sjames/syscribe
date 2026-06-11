---
type: CalculationDef
name: RangeMoE
custom_fields:
  mg_moe: true
  mg_moe_measures: REQ-MG-RANGE-033
  mg_moe_unit: km
  mg_moe_direction: maximize
  mg_moe_threshold: 10
  mg_moe_objective: 5
---

For a maximize MoE the objective (target) must be at least the threshold
(minimum acceptable). Here objective 5 < threshold 10 → MG033 under the gate.
