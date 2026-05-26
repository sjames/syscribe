---
type: Diagram
name: MissionExecutionSeq
diagramKind: Sequence
svgMode: companion
svgFile: ./MissionExecutionSeq.svg
subject: Behavior::MissionExecution
shapes:
  ll-gcs:
    ref: GroundStation::GroundControlStation
    kind: actor
  ll-fc:
    ref: UAV::Avionics::FlightController
    kind: lifeline
  ll-prop:
    ref: UAV::Propulsion::PropulsionSystem
    kind: lifeline
  ll-gps:
    ref: UAV::Avionics::GPSReceiver
    kind: lifeline
  act-fc:
    ref: UAV::Avionics::FlightController
    kind: activation
    parent: ll-fc
  act-prop-takeoff:
    ref: UAV::Propulsion::PropulsionSystem
    kind: activation
    parent: ll-prop
  act-prop-land:
    ref: UAV::Propulsion::PropulsionSystem
    kind: activation
    parent: ll-prop
  frag-takeoff:
    ref: Behavior::TakeoffAction
    kind: fragment
  frag-weather:
    ref: Behavior::MissionExecution
    kind: fragment
  frag-nav:
    ref: Behavior::WaypointNavAction
    kind: fragment
  frag-land:
    ref: Behavior::LandingAction
    kind: fragment
edges:
  e-start:
    ref: Behavior::MissionExecution
    source: ll-gcs
    target: ll-fc
    kind: message
  e-throttle:
    ref: Behavior::TakeoffAction::setThrottle
    source: ll-fc
    target: ll-prop
    kind: message
  e-alt-telem-1:
    ref: Behavior::TakeoffAction::waitForAltitude
    source: ll-prop
    target: ll-fc
    kind: message
  e-req-fix-1:
    ref: Behavior::WaypointNavAction
    source: ll-fc
    target: ll-gps
    kind: message
  e-gps-fix-1:
    ref: Behavior::WaypointNavAction::navigateWaypoints
    source: ll-gps
    target: ll-fc
    kind: return
  e-abort:
    ref: Behavior::MissionExecution::abortMission
    source: ll-fc
    target: ll-gcs
    kind: message
  e-req-fix-2:
    ref: Behavior::WaypointNavAction::navigateWaypoints
    source: ll-fc
    target: ll-gps
    kind: message
  e-gps-fix-2:
    ref: Behavior::WaypointNavAction::awaitArrival
    source: ll-gps
    target: ll-fc
    kind: return
  e-descent:
    ref: Behavior::LandingAction::commandDescent
    source: ll-fc
    target: ll-prop
    kind: message
  e-alt-telem-2:
    ref: Behavior::LandingAction::waitForGround
    source: ll-prop
    target: ll-fc
    kind: message
  e-disarm:
    ref: Behavior::LandingAction::disarm
    source: ll-fc
    target: ll-prop
    kind: message
  e-disarmed:
    ref: Behavior::LandingAction
    source: ll-prop
    target: ll-fc
    kind: return
  e-complete:
    ref: Behavior::MissionExecution
    source: ll-fc
    target: ll-gcs
    kind: return
---

<img src="./MissionExecutionSeq.svg" alt="Mission Execution Sequence Diagram" width="100%"/>

Sequence diagram for `MissionExecution`. Shows the message flow between the ground station, flight controller, propulsion system, and GPS receiver across the three sub-actions:

- **TakeoffAction** — FC commands initial throttle; loop monitors altitude telemetry until target altitude is reached.
- **Weather check** — `alt` fragment: if wind > 12 m/s the FC sends `abortMission` to GCS; else waypoint navigation begins.
- **WaypointNavAction** — loop requests GPS fixes and advances through each waypoint.
- **LandingAction** — FC commands descent; loop monitors altitude until touchdown; motors disarmed; `missionComplete` returned to GCS.
