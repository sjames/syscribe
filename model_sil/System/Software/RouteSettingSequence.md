---
type: ActionDef
name: Route Setting Sequence
domain: software
successionConnections:
  - after: requestRoute
    before: checkConflicts
    guard: "signaller.routeRequest.isValid"
  - after: checkConflicts
    before: lockPoints
    guard: "conflictChecker.routeIsClear"
  - after: lockPoints
    before: verifyPointsPosition
    guard: "pointsController.commandAccepted"
  - after: verifyPointsPosition
    before: clearSignal
    guard: "pointsController.detectionConfirmed"
---

Ordered sequence of vital actions executed by the vital processor to set a train route in response to a signaller request. Each step is gated by a vital condition; failure at any step immediately halts the sequence and the protecting signal remains at the most-restrictive aspect (red).

**Step 1 — requestRoute.** The signaller submits a route request identifying the entry and exit signals. The request is validated for syntactic and logical correctness (valid route identifiers, valid signaller credentials).

**Step 2 — checkConflicts.** The ConflictChecker is consulted to verify that the requested route does not conflict with any currently-set or in-progress route. The check must return "route is clear" within the same scan cycle.

**Step 3 — lockPoints.** The PointsController is instructed to lock and, if necessary, move all points required for the route to their correct positions. The command is accepted only if all points are either already in the correct position (detected) or free to move (not locked by another route).

**Step 4 — verifyPointsPosition.** The PointsController waits for all commanded points to reach and hold their detected positions. This step spans one or more scan cycles up to the moveTimeoutMs limit. The sequence does not advance until all points report confirmed detection.

**Step 5 — clearSignal.** The SignalController evaluates all clearing conditions (track circuit status, points detection, level crossing barriers) and, if all are satisfied, commands the protecting signal to the appropriate non-red aspect.

The sequence is formally specified as a B-Method operation with pre- and postconditions. Pre-conditions assert the initial state invariants; postconditions assert that the route is locked and the signal is cleared only when all conditions are met.
