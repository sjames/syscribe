---
id: SG-SIL-001
type: SafetyGoal
status: approved
title: Prevent conflicting train routes from being set simultaneously
silLevel: 4
safeState: All signals to most-restrictive aspect (red); all set routes cancelled
ftti: 50ms
hazardousEvents:
  - HE-SIL-001
  - HE-SIL-002
---

The interlocking **shall** ensure that no two conflicting routes can be simultaneously set or partially set. A conflict exists when two routes share any track section, any points machine, or any defined protection zone. The safe state is achieved within one vital scan cycle (≤50 ms) by the 2oo2 cross-comparison detecting a discrepancy and both channels asserting the fail-safe relay chain simultaneously.

**Scope of protection.** This goal covers both the initial route-setting decision (proactive prevention — the ConflictChecker must deny any route-setting request that conflicts with a currently set route) and the continuous supervision obligation (reactive protection — if a route becomes conflicting due to a system state change after setting, the SignalController must immediately revert all affected signals to the most-restrictive aspect).

**Integrity requirements.** This goal requires SIL 4 integrity for both the ConflictChecker (System::Software::ConflictChecker) and the RouteProcessor (System::Software::RouteProcessor), since both are in the execution path of the conflict detection and signal clearance decision. The conflict matrix must be formally verified against the track layout using a certified model checker or formal proof tool (EN 50128 §6.7.4.2 — Formal Methods).

**Safe state definition.** "Most-restrictive aspect" means a red signal (stop aspect) for all affected signals. A cancelled route releases all route-locks and points reservations and allows points to move to default positions. The safe-state transition must be completed within one vital scan cycle (≤50 ms) measured from the instant the 2oo2 comparison detects a discrepancy.

**Fault tolerance time interval (FTTI): 50 ms.** This is derived from the maximum time before a hazardous condition becomes uncontrollable — specifically, the time from a conflicting route being set to a train passing a signal in the wrong direction at maximum permitted speed. At 200 km/h the train travels ~2.8 m in 50 ms, well within the overlap provision.
