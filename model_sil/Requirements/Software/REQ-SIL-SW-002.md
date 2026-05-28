---
type: Requirement
id: REQ-SIL-SW-002
title: Vital software shall be developed by two independent diverse teams
status: approved
reqDomain: software
silLevel: 4
verificationMethod: inspection
derivedFrom:
  - REQ-SIL-SAFE-000
breakdownAdr: ADR-SIL-SW-001
---

The vital (SIL 4) software for Channel A and Channel B **shall** be developed by two completely independent development teams with no shared personnel, tools, or development infrastructure. The two implementations **shall** use different implementation languages (e.g., Ada and C with MISRA subset), different compilers, and different RTOS configurations. Both implementations **shall** be derived from the same B-Method formal specification (REQ-SIL-SW-003) but **shall** independently translate the formal specification to code without sharing intermediate artefacts. The independence of the two teams **shall** be verified by the Independent Safety Assessor (ISA) during the development lifecycle. Purpose: to defeat common-cause software failure — the dominant residual risk at SIL 4. Reference: EN 50128 §6.7.4 (diverse programming).
