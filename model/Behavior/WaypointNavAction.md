---
type: ActionDef
name: WaypointNavAction
supertype: Actions::Action
parameters:
  - name: waypoints
    typedBy: Items::GPSFix
    direction: in
    multiplicity: "1..*"
    isOrdered: true
  - name: success
    typedBy: ScalarValues::Boolean
    direction: return
subActions:
  - name: navigateWaypoints
    kind: LoopAction
    loopKind: for
    variable: waypoint
    sequence: "waypoints"
    body:
      - name: awaitArrival
        kind: AcceptAction
        payload: Items::GPSFix
        trigger:
          kind: change
          condition: "currentFix.distanceTo(waypoint) < 2.0"
successionConnections:
  - after: navigateWaypoints
    before: navigateWaypoints
---

Waypoint navigation action iterating through an ordered list of GPS waypoints, advancing to each successive point upon arrival within 2 m.
