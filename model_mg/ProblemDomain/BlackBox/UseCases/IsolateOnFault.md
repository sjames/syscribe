---
type: UseCaseDef
name: IsolateOnFault
subject: ProblemDomain::BlackBox::SystemContext::ChargingStation
actors:
  - ProblemDomain::BlackBox::SystemContext::ElectricVehicle
  - ProblemDomain::BlackBox::SystemContext::PowerGrid
refines:
  - ProblemDomain::BlackBox::StakeholderNeeds::Safe
custom_fields:
  mg_cell: B2
objectives:
  - "Detect an electrical fault and isolate energy safely"
---

On detecting an insulation, over-current or connector fault, the station opens
its contactors and de-energises the connector within the safety reaction time.
