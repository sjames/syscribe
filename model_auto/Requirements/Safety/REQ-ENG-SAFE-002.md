---
type: Requirement
id: REQ-ENG-SAFE-002
name: Hardware watchdog shall reset ECU within 30 ms of software failure
status: approved
reqDomain: hardware
asilLevel: D
verificationMethod: test
derivedFrom:
  - REQ-ENG-SAFE-000
breakdownAdr: ADR-ENG-SAFE-001
derivedFromSafetyGoal: SG-ENG-001
---

The external hardware watchdog timer **shall** detect software failure (failure
to service within the 10 ms window) and assert a hardware reset of the
microcontroller within 30 ms of the missed service window, independent of
the ECU software state.

The watchdog **shall** operate in windowed mode to also detect software that
services the watchdog too frequently (indicative of a stuck loop).
