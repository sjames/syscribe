---
type: Allocation
name: FunctionAllocation
features:
  - name: takeoffToFC
    type: Allocation
    allocatedFrom: Behavior::TakeoffAction
    allocatedTo: UAV::Avionics::FlightController
  - name: landingToFC
    type: Allocation
    allocatedFrom: Behavior::LandingAction
    allocatedTo: UAV::Avionics::FlightController
  - name: waypointNavToFC
    type: Allocation
    allocatedFrom: Behavior::WaypointNavAction
    allocatedTo: UAV::Avionics::FlightController
  - name: missionExecToFC
    type: Allocation
    allocatedFrom: Behavior::MissionExecution
    allocatedTo: UAV::Avionics::FlightController
---

Functional allocation mapping all behavior actions to the flight controller hardware element.
