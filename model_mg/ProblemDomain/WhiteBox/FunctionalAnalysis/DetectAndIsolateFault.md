---
type: ActionDef
name: DetectAndIsolateFault
allocatedTo: ProblemDomain::WhiteBox::LogicalSubsystems::SafetyInterlockSubsystem
parameters:
  - name: insulationOhms
    typedBy: ScalarValues::Real
    direction: in
  - name: residualCurrent
    typedBy: ScalarValues::Real
    direction: in
refines:
  - ProblemDomain::WhiteBox::SystemRequirements::SafetyIsolation
custom_fields:
  mg_cell: W2
---

Functional action: continuously monitor insulation resistance and residual
current; on a fault, command the contactors open and de-energise the connector.
