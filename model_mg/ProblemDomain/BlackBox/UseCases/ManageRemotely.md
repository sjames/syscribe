---
type: UseCaseDef
name: ManageRemotely
subject: ProblemDomain::BlackBox::SystemContext::ChargingStation
actors:
  - ProblemDomain::BlackBox::SystemContext::BackOfficeCloud
refines:
  - ProblemDomain::BlackBox::StakeholderNeeds::Reliable
custom_fields:
  mg_cell: B2
objectives:
  - "Monitor health and apply updates remotely"
---

The back-office cloud monitors stall health, raises maintenance alerts and
applies over-the-air firmware updates to keep availability high.
