---
type: Requirement
id: REQ-EVCS-SYS-004
title: "The station shall keep amortised cost per session at or below USD 4.00"
status: approved
requirementKind: system
reqDomain: system
verificationMethod: analysis
derivedFrom:
  - ProblemDomain::BlackBox::StakeholderNeeds::Affordable
breakdownAdr: Decisions::ADR_Architecture
custom_fields:
  mg_cell: W1
---

The amortised capital-plus-energy cost per completed charging session shall not
exceed USD 4.00 at the design utilisation, so the station is profitable at
competitive retail pricing.
