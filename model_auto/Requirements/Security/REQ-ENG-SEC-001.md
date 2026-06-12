---
type: Requirement
id: REQ-ENG-SEC-001
name: Safety-critical CAN messages shall be authenticated using MAC
status: approved
reqDomain: software
verificationMethod: test
derivedFrom:
  - REQ-ENG-SYS-000
breakdownAdr: ADR-ENG-SYS-001
derivedFromSecurityGoal: CSG-ENG-001
---

The Engine ECU **shall** authenticate all safety-critical outbound and inbound
CAN messages using a message authentication code (AUTOSAR SecOC, CMAC-AES-128,
24-bit truncated) with a freshness counter. Any received message with an invalid
MAC **shall** be rejected and a security diagnostic trouble code (DTC) shall be
set within 50 ms of detection.
