---
type: UseCaseDef
name: ChargeVehicle
subject: ProblemDomain::BlackBox::SystemContext::ChargingStation
actors:
  - ProblemDomain::BlackBox::SystemContext::EVDriver
  - ProblemDomain::BlackBox::SystemContext::ElectricVehicle
refines:
  - ProblemDomain::BlackBox::StakeholderNeeds::FastTurnaround
custom_fields:
  mg_cell: B2
objectives:
  - "Deliver a fast DC charge to a connected EV"
---

A driver connects their EV, the station negotiates a charging contract with the
vehicle and delivers DC energy until the target state of charge is reached or
the driver stops the session.
