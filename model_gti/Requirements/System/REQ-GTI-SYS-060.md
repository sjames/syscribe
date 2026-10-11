---
type: Requirement
id: REQ-GTI-SYS-060
name: "Region selection by commissioning configuration"
status: approved
reqDomain: system
verificationMethod: test
derivedFrom:
  - REQ-GTI-STK-006
  - REQ-GTI-STK-008
breakdownAdr: ADR-GTI-REQ-001
derivedFromCybersecurityGoal: CSG-GTI-001
calLevel: CAL4
---

The region and its grid profile **shall** be selected by an authenticated commissioning action, not by a different firmware image; after commissioning the region **shall** only be changeable with a service credential and **shall** be logged.
