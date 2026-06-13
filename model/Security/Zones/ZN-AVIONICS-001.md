---
type: Zone
id: ZN-AVIONICS-001
name: Avionics Control Zone
status: approved
targetSL: 3
achievedSL: 3
members:
  - UAV::Avionics::FlightController
  - UAV::Avionics::GPSReceiver
rationale: >
  Hosts the flight-critical control functions; SL 3 required (loss of integrity
  could cause loss of the aircraft).
---
The avionics control zone hosts the flight controller and primary sensors.
