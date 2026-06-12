---
type: Requirement
id: REQ-SIL-SAFE-000
name: CBI shall implement safety instrumented functions achieving SIL 4 for all identified hazards
status: approved
reqDomain: system
derivedFrom:
  - REQ-SIL-SYS-000
breakdownAdr: ADR-SIL-SYS-001
---

All Safety Instrumented Functions (SIFs) identified in the HARA (`Safety::HARA`) shall be implemented with integrity sufficient to meet their assigned SIL targets. SIL 4 functions shall achieve dangerous undetected failure rate < 10⁻⁸ /h per function. SIL 3 functions: < 10⁻⁷ /h. Safe-side failures shall be reported to the maintainer but shall not be treated as safety failures.
