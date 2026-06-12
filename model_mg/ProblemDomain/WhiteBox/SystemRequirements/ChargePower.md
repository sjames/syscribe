---
type: Requirement
id: REQ-EVCS-SYS-001
name: "The station shall deliver at least 150 kW DC to a single connected vehicle"
status: approved
requirementKind: system
reqDomain: system
verificationMethod: test
derivedFrom:
  - ProblemDomain::BlackBox::StakeholderNeeds::FastTurnaround
breakdownAdr: Decisions::ADR_Architecture
custom_fields:
  mg_cell: W1
---

The station shall deliver a continuous DC output power of at least 150 kW to a
single connected, compatible electric vehicle under nominal grid and thermal
conditions.
