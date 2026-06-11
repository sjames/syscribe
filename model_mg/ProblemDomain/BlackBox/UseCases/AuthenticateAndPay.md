---
type: UseCaseDef
name: AuthenticateAndPay
subject: ProblemDomain::BlackBox::SystemContext::ChargingStation
actors:
  - ProblemDomain::BlackBox::SystemContext::EVDriver
  - ProblemDomain::BlackBox::SystemContext::BackOfficeCloud
refines:
  - ProblemDomain::BlackBox::StakeholderNeeds::Affordable
custom_fields:
  mg_cell: B2
objectives:
  - "Authenticate the driver and bill the completed session"
---

The driver authenticates (RFID/app), the station obtains authorisation from the
back-office cloud, and on completion the metered energy is billed to the
driver's account.
