---
type: Requirement
id: REQ-SIL-SAFE-003
name: Points controller shall verify detected position before reporting confirmation
status: approved
reqDomain: software
silLevel: 4
verificationMethod: test
derivedFrom:
  - REQ-SIL-SAFE-000
breakdownAdr: ADR-SIL-SYS-001
derivedFromSafetyGoal: SG-SIL-002
---

The PointsController **shall** only report a "position confirmed" state to the RouteProcessor when the detection contacts (independent from the drive circuit) confirm the commanded position. Detection **shall** be a positively-proved condition — the presence of a detection current, not the absence of an alarm. After issuing a move command, the PointsController **shall** wait up to the configured `moveTimeoutMs` for confirmation; if confirmation is not received within this window, the controller **shall** report "position unconfirmed" and the route processor **shall** not permit signal clearance. The PointsController **shall** continuously monitor detection during route occupancy; any loss of detection while a signal is clear for that route **shall** immediately cause the SignalController to return the signal to most-restrictive aspect.
