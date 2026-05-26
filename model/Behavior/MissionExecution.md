---
type: ActionDef
name: MissionExecution
supertype: Actions::Action
parameters:
  - name: missionWaypoints
    typedBy: Items::GPSFix
    direction: in
    multiplicity: "1..*"
    isOrdered: true
  - name: missionSuccess
    typedBy: ScalarValues::Boolean
    direction: return
subActions:
  - name: takeoff
    kind: PerformAction
    typedBy: Behavior::TakeoffAction
  - name: navigate
    kind: PerformAction
    typedBy: Behavior::WaypointNavAction
  - name: land
    kind: PerformAction
    typedBy: Behavior::LandingAction
  - name: checkWeather
    kind: IfAction
    condition: "windSpeedMs > 12.0"
    then:
      - name: abortMission
        kind: SendAction
        payload: Items::ControlCommand
        via: controlOut
    else:
      - name: continueMission
        kind: PerformAction
        typedBy: Behavior::WaypointNavAction
controlNodes:
  - name: missionStart
    kind: ForkNode
  - name: missionEnd
    kind: JoinNode
successionConnections:
  - after: missionStart
    before: takeoff
  - after: takeoff
    before: checkWeather
  - after: checkWeather
    before: navigate
  - after: navigate
    before: land
  - after: land
    before: missionEnd
---

Top-level mission execution action orchestrating takeoff, waypoint navigation, weather check, and landing. Aborts if wind speed exceeds safe limits.
