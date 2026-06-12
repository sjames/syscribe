---
type: Requirement
id: REQ-EVCS-SYS-002
name: "The station shall achieve at least 99.0% stall availability over a rolling year"
status: approved
requirementKind: system
reqDomain: system
verificationMethod: analysis
derivedFrom:
  - ProblemDomain::BlackBox::StakeholderNeeds::Reliable
breakdownAdr: Decisions::ADR_Architecture
custom_fields:
  mg_cell: W1
---

Each dispenser stall shall be available for charging at least 99.0% of the time,
measured over a rolling 12-month window, excluding scheduled maintenance.
