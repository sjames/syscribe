---
type: UseCaseDef
name: AutonomousMissionUseCase
supertype: UseCases::UseCase
features:
  - name: subject
    typedBy: UAV::UAVSystem
    direction: in
  - name: actor
    typedBy: GroundStation::OperatorConsole
    direction: in
actors:
  - GroundStation::OperatorConsole
extensionPoints:
  - name: payloadActivation
    description: "Point at which payload recording can be activated or deactivated"
  - name: emergencyOverride
    description: "Point at which operator can assert manual override"
steps:
  - "Operator uploads mission waypoints via ground control station"
  - "UAV arms and runs pre-flight checks"
  - "UAV executes autonomous takeoff"
  - "UAV navigates waypoints using GPS guidance"
  - "UAV returns to home and lands autonomously"
  - "Operator confirms landing and disarms"
---

Primary nominal mission use case for fully autonomous waypoint flight.
