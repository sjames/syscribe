---
type: ActionDef
name: RegulateDcOutput
allocatedTo: ProblemDomain::WhiteBox::LogicalSubsystems::PowerConversionSubsystem
parameters:
  - name: setpoint
    typedBy: ScalarValues::Real
    direction: in
  - name: measuredCurrent
    typedBy: ScalarValues::Real
    direction: in
refines:
  - ProblemDomain::WhiteBox::SystemRequirements::ChargePower
custom_fields:
  mg_cell: W2
---

Functional action: close the current/voltage control loop to track the agreed
setpoint while respecting thermal and grid-import limits.
