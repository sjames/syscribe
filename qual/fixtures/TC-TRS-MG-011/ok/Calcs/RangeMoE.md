---
type: CalculationDef
name: RangeMoE
features:
  - name: batteryKWh
    typedBy: ScalarValues::Real
    direction: in
  - name: rangeKm
    typedBy: ScalarValues::Real
    direction: out
    isDerived: true
expression: "rangeKm = batteryKWh * 5"
custom_fields:
  mg_moe: true
  mg_moe_measures: REQ-MG-RANGE-001
  mg_moe_unit: km
  mg_moe_direction: maximize
  mg_moe_threshold: 10
  mg_moe_objective: 100
---

A measure of effectiveness whose expression uses the bound variable batteryKWh,
resolved from each variant Configuration's parameterBindings (final-segment match
on the binding key). The trade study scores each variant from this expression.
