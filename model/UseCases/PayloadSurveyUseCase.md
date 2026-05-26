---
type: UseCaseDef
name: PayloadSurveyUseCase
supertype: UseCases::UseCase
features:
  - name: subject
    typedBy: UAV::UAVSystem
    direction: in
  - name: payload
    typedBy: UAV::Payload::Camera
    direction: in
actors:
  - GroundStation::OperatorConsole
extends:
  - target: UseCases::AutonomousMissionUseCase
    extensionPoint: payloadActivation
    condition: "altitude >= 30m and flightMode == hover"
steps:
  - "UAV enters survey grid waypoint pattern"
  - "Camera activates at each grid point"
  - "Imagery is tagged with GPS coordinates"
  - "Telemetry confirms each capture to GCS"
  - "UAV completes grid and returns to home"
---

Aerial survey extension use case for photogrammetry mission profiles.
