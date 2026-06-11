---
type: CalculationDef
name: AvailabilityMoE
returnType: ScalarValues::Real
expression: "moe = baseAvailability + redundancySpare * 0.005"
custom_fields:
  mg_cell: B4
  mg_moe: true
  mg_moe_measures: ProblemDomain::WhiteBox::SystemRequirements::Availability
  mg_moe_unit: fraction
  mg_moe_direction: maximize
  mg_moe_threshold: 0.99
  mg_moe_objective: 0.999
  mg_moe_weight: 0.25
---

**MoE — Stall Availability.** Effectiveness at the reliability need: base
availability uplifted by N+ spare converter modules (each spare adds 0.5%).
Maximised; below 0.99 is a knock-out.
