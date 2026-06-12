---
type: PartDef
name: EnergyControlSubsystem
allocatedTo: SolutionDomain::PhysicalComponents::ChargeController
domain: system
custom_fields:
  mg_cell: W3
  mg_layer: logical
satisfies:
  - ProblemDomain::WhiteBox::SystemRequirements::ChargePower
---

Logical subsystem that runs the charging session state machine, negotiates the contract with the vehicle and commands the power conversion.
