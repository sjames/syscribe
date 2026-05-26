---
type: UseCaseDef
name: EmergencyLandingUseCase
supertype: UseCases::UseCase
features:
  - name: subject
    typedBy: UAV::UAVSystem
    direction: in
actors:
  - GroundStation::OperatorConsole
extends:
  - target: UseCases::AutonomousMissionUseCase
    extensionPoint: emergencyOverride
    condition: "battery < 15% or link-loss > 5s"
steps:
  - "Emergency condition detected (battery critical or link loss)"
  - "Flight controller transitions to fault state"
  - "Autonomous safe landing initiated"
  - "UAV descends at ≤3 m/s to landing site"
  - "UAV disarms on touchdown"
---

Extension of the nominal mission use case for contingency emergency landing scenarios.
