---
type: Requirement
id: REQ-SIL-SAFE-002
name: Signal controller shall only clear a signal when all interlocking conditions are simultaneously verified
status: approved
reqDomain: software
silLevel: 4
verificationMethod: analysis
derivedFrom:
  - REQ-SIL-SAFE-000
breakdownAdr: ADR-SIL-SYS-001
derivedFromSafetyGoal: SG-SIL-002
---

The SignalController **shall** evaluate the following conditions atomically on each scan cycle, and shall only output a "clear" aspect if all conditions are simultaneously true:

1. Route is set and locked by the RouteProcessor.
2. All track circuit sections on the route and the defined overlap are reporting "clear".
3. All points on the route are confirmed in the correct position by the PointsController.
4. No conflicting route is active (confirmed by ConflictChecker).
5. For routes including a level crossing, the LevelCrossingModule reports barriers confirmed down.

Any condition becoming false while a signal is clear **shall** cause an immediate return to the most-restrictive aspect within one vital scan cycle (≤50ms). "Immediately" means without requiring a new signaller request — the transition to red is automatic and automatic re-clearance is not permitted. Implementation shall be formally specified per REQ-SIL-SW-003 and verified by test TC-SIL-SAFE-002.
