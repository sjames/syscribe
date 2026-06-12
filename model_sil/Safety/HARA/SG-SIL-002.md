---
id: SG-SIL-002
type: SafetyGoal
status: approved
name: Prevent signal clearance unless all route conditions are verified
silLevel: 4
safeState: Signal returns to most-restrictive aspect within one scan cycle
ftti: 50ms
hazardousEvents:
  - HE-SIL-001
  - HE-SIL-003
---

The SignalController (System::Software::SignalController) **shall** only clear a signal when all interlocking conditions are simultaneously satisfied:

1. Route set and locked by the RouteProcessor.
2. All track circuits in the route and overlap confirmed clear by the TrackCircuitInterface.
3. All points in the route detected in the correct position by the PointsDriveModule.
4. No overlapping or conflicting routes active (verified by ConflictChecker).
5. Any applicable level crossing barriers confirmed down (verified by LevelCrossingModule).

**Continuous supervision obligation.** Any single condition becoming false while a signal is clear shall cause an immediate return to the most-restrictive aspect within one vital scan cycle (≤50 ms). The interlocking does not wait for a full re-evaluation — a single condition failure triggers an unconditional revert. This is the "fail-safe" principle applied to output state maintenance.

**Proactive vs. reactive protection.** This goal covers both:
- *Proactive prevention*: the pre-clearance check verifies all conditions before the signal is first cleared.
- *Reactive protection*: the continuous supervision loop monitors all conditions on every scan cycle and reverts the signal immediately if any condition becomes false.

**Integrity requirements.** SIL 4 is required for the SignalController software and the VitalProcessor hardware executing it. The track circuit interface (TrackCircuitInterface) and points drive/detection module (PointsDriveModule) must each meet SIL 4 dangerous failure rate targets as they are in the detection path for two of the five conditions.

**Fault tolerance time interval (FTTI): 50 ms.** Derived from the maximum time before a train passes the signal at line speed after an incorrect clearance. The 50 ms bound matches SG-SIL-001 since both goals share the same scan cycle constraint.
