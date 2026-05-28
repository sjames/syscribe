---
type: Requirement
id: REQ-SIL-SAFE-004
title: Signal clearance shall be conditional on level crossing barriers confirmed in lowered position
status: approved
reqDomain: software
silLevel: 3
verificationMethod: test
derivedFrom:
  - REQ-SIL-SAFE-000
breakdownAdr: ADR-SIL-SYS-001
derivedFromSafetyGoal: SG-SIL-003
---

For any route that includes or approaches a level crossing, the SignalController **shall** include the LevelCrossingModule's "barriers confirmed down" status as a mandatory precondition for signal clearance. The LevelCrossingModule **shall** report barriers confirmed down only when both primary and secondary barrier detection circuits confirm the lowered position. The minimum road warning time (from barrier start to signal clearance) **shall** be configurable per crossing and **shall** default to 30 seconds (BR standard minimum). If barrier confirmation is lost while a signal is clear, the signal **shall** revert to most-restrictive aspect within one scan cycle.
