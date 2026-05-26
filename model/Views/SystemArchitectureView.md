---
type: View
name: SystemArchitectureView
viewpoint: Viewpoints::SystemsEngineerViewpoint
expose:
  - UAV::UAVSystem
  - UAV::Airframe
  - UAV::Propulsion::PropulsionSystem
  - UAV::Avionics::AvionicsBay
  - UAV::Power::PowerSystem
  - UAV::Payload::PayloadBay
  - GroundStation::GroundControlStation
---

Top-level system architecture view exposing the main structural decomposition of the UAV platform.
