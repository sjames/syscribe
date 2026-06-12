---
type: Requirement
id: REQ-EVCS-SYS-003
name: "The station shall de-energise the connector within 100 ms of detecting a fault"
status: approved
requirementKind: system
reqDomain: system
verificationMethod: test
derivedFrom:
  - ProblemDomain::BlackBox::StakeholderNeeds::Safe
breakdownAdr: Decisions::ADR_Architecture
custom_fields:
  mg_cell: W1
---

On detection of an insulation, residual-current or over-current fault, the
station shall open the DC contactors and reduce the connector voltage below the
touch-safe threshold within 100 ms.
