---
type: PartDef
name: PowerConversionSubsystem
allocatedTo: SolutionDomain::PhysicalComponents::PowerCabinet
domain: system
custom_fields:
  mg_cell: W3
  mg_layer: logical
satisfies:
  - ProblemDomain::WhiteBox::SystemRequirements::ChargePower
  - ProblemDomain::WhiteBox::SystemRequirements::SessionCost
---

Logical subsystem that converts grid AC to regulated DC and tracks the agreed charging setpoint.
