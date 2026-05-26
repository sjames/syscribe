---
type: Allocation
name: RequirementAllocation
features:
  - name: flightDurationToBattery
    type: Allocation
    allocatedFrom: Requirements::FlightDurationReq
    allocatedTo: UAV::Power::BatteryPack
  - name: massReqToSystem
    type: Allocation
    allocatedFrom: Requirements::MaxTakeoffMassReq
    allocatedTo: UAV::UAVSystem
  - name: safetyFCReqToFC
    type: Allocation
    allocatedFrom: Requirements::FaultTolerantFCReq
    allocatedTo: UAV::Avionics::FlightController
  - name: landingReqToFC
    type: Allocation
    allocatedFrom: Requirements::SafeLandingReq
    allocatedTo: UAV::Avionics::FlightController
  - name: navAccuracyToGPS
    type: Allocation
    allocatedFrom: Requirements::PositionAccuracyReq
    allocatedTo: UAV::Avionics::GPSReceiver
  - name: dataLinkToFC
    type: Allocation
    allocatedFrom: Requirements::DataLinkReq
    allocatedTo: UAV::Avionics::FlightController
---

Requirement-to-component allocation tracing each system requirement to its responsible hardware element.
Requirements are now in native Requirement format with stable REQ-* IDs; the allocatedFrom references
use qualified names (resolved by the model root path).
