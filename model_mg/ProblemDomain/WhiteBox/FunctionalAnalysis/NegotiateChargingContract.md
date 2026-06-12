---
type: ActionDef
name: NegotiateChargingContract
allocatedTo: ProblemDomain::WhiteBox::LogicalSubsystems::EnergyControlSubsystem
parameters:
  - name: vehicleMaxCurrent
    typedBy: ScalarValues::Real
    direction: in
  - name: targetSoc
    typedBy: ScalarValues::Real
    direction: in
  - name: agreedSetpoint
    typedBy: ScalarValues::Real
    direction: return
refines:
  - ProblemDomain::WhiteBox::SystemRequirements::ChargePower
custom_fields:
  mg_cell: W2
---

Functional action: negotiate the charging contract with the vehicle over the
control pilot — exchange limits and the target state of charge, and return the
agreed current/voltage setpoint.
