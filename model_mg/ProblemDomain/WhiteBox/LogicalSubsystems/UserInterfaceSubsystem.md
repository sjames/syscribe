---
type: PartDef
name: UserInterfaceSubsystem
domain: system
custom_fields:
  mg_cell: W3
  mg_layer: logical
satisfies:
  - ProblemDomain::WhiteBox::SystemRequirements::Availability
---

Logical subsystem that handles driver authentication, session start/stop and on-stall display.
