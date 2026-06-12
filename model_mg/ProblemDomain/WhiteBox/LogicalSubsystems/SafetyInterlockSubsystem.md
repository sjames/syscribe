---
type: PartDef
name: SafetyInterlockSubsystem
allocatedTo: SolutionDomain::PhysicalComponents::SafetyMonitorUnit
domain: system
custom_fields:
  mg_cell: W3
  mg_layer: logical
satisfies:
  - ProblemDomain::WhiteBox::SystemRequirements::SafetyIsolation
---

Logical subsystem that monitors insulation and residual current and isolates energy on fault.
