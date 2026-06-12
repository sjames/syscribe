---
type: PartDef
name: UserInterfaceSubsystem
allocatedTo: SolutionDomain::PhysicalComponents::DispenserHmi
domain: system
custom_fields:
  mg_cell: W3
  mg_layer: logical
satisfies:
  - ProblemDomain::WhiteBox::SystemRequirements::Availability
---

Logical subsystem that handles driver authentication, session start/stop and on-stall display.
