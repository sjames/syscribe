---
type: PartDef
name: ConnectivitySubsystem
allocatedTo: SolutionDomain::PhysicalComponents::NetworkGateway
domain: system
custom_fields:
  mg_cell: W3
  mg_layer: logical
satisfies:
  - ProblemDomain::WhiteBox::SystemRequirements::Availability
---

Logical subsystem that links the station to the back-office cloud for authorisation, billing and remote management.
