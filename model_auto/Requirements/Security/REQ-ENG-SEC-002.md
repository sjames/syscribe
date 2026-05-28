---
type: Requirement
id: REQ-ENG-SEC-002
title: ECU calibration programming sessions shall require cryptographic authentication
status: approved
reqDomain: software
verificationMethod: test
derivedFrom:
  - REQ-ENG-SYS-000
breakdownAdr: ADR-ENG-SYS-001
derivedFromSecurityGoal: CSG-ENG-002
---

The Engine ECU **shall** require a cryptographic challenge-response authentication
(seed-and-key, minimum 128-bit security level) before granting a UDS security
access level that permits ECU calibration reprogramming. Any programming attempt
that fails authentication **shall** be rejected and a security DTC set within
200 ms. After three consecutive failures, the ECU **shall** impose a 10-minute
lockout.
